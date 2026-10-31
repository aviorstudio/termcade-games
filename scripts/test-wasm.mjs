// Exercise the shipping Rust WASI guests through the public nine-function ABI.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { WASI } from 'node:wasi';
import { join } from 'node:path';

const target = process.env.CARGO_TARGET_DIR || 'target';
const games = [['asteroid',72],['tetris',32],['brickough',64]];
const names = ['termcade_abi','termcade_init','termcade_playfield','termcade_reset','termcade_key','termcade_update','termcade_draw','termcade_score','termcade_hud'];
for (const [game,width] of games) {
  const module = new WebAssembly.Module(readFileSync(join(target,'wasm32-wasip1','release',`termcade_${game}.wasm`)));
  for (const entry of WebAssembly.Module.imports(module)) assert.equal(entry.module,'wasi_snapshot_preview1');
  for (const [cols,rows] of [[1,2],[2,2],[2,3],[3,3]]) {
    const wasi = new WASI({ version:'preview1', args:[], env:{}, preopens:{}, returnOnExit:true });
    const instance = new WebAssembly.Instance(module,wasi.getImportObject());
    wasi.initialize(instance);
    const e=instance.exports;
    for (const name of names) assert.equal(typeof e[name],'function',name);
    assert.equal(e.termcade_abi(),1);
    const pointer=e.termcade_init(cols,rows);
    e.termcade_reset();
    const field=e.termcade_playfield();
    assert.equal(Number(field>>32n),width);
    assert.equal(Number(field&0xffffffffn),40);
    const pixelCount=width*cols*40*rows/2;
    assert.equal(Number(pointer&0xffffffffn),pixelCount*4);
    const framebuffer=Number(pointer>>32n);
    assert.ok(framebuffer>=0&&framebuffer+pixelCount*4<=e.memory.buffer.byteLength);
    assert.equal(e.termcade_score(),0n);
    e.termcade_draw();
    assert.ok(new Uint32Array(e.memory.buffer,framebuffer,pixelCount).some(value=>value!==0));
    const readHud=()=>{
      const packed=e.termcade_hud();const offset=Number(packed>>32n),length=Number(packed&0xffffffffn);
      assert.ok(length>0&&offset>=0&&offset+length<=e.memory.buffer.byteLength);
      return JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(e.memory.buffer,offset,length)));
    };
    assert.equal(readHud().Fields[0].Value,'000000');
    for (const key of [1,2,3,4,5,6,7]) {e.termcade_key(key,1);assert.ok([0,1].includes(e.termcade_update()));e.termcade_key(key,0);}
    for (let tick=0;tick<180;tick++) {const status=e.termcade_update();assert.ok([0,1].includes(status));e.termcade_draw();readHud();if(status===1)break;}
    if(game==='tetris') assert.ok(e.termcade_score()>0n,'hard drop must score');
    e.termcade_reset();assert.equal(e.termcade_score(),0n);e.termcade_draw();assert.equal(readHud().Fields[0].Value,'000000');
    assert.ok(e.memory.buffer.byteLength<=64*1024*1024);
    console.log(`${game}: ${cols}x${rows} ABI, pixels, key transitions, 180 ticks, HUD and reset passed`);
  }
}
