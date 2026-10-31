// Independent snapshots recorded from the original Go games at d84aaf6.
// Replays compare simulation results and all raw pixels, before terminal glyph
// reduction, at nine checkpoints in each of eight seeded/style scenarios.
fn replay<G: termcade_sdk::Game>(fixture: &str, factory: impl Fn(i64)->G) {
    use sha2::{Digest,Sha256};
    use termcade_sdk::{Canvas, CellShape, Key, Status, BLACK};
    let traces:serde_json::Value=serde_json::from_str(fixture).unwrap();
    for trace in traces.as_array().unwrap() {
        let seed=trace["seed"].as_i64().unwrap();
        let shape_name=trace["shape"].as_str().unwrap();
        let shape=CellShape::lookup(shape_name).expect("fixture style");
        let mut g=factory(seed);let info=g.info();
        let mut canvas=Canvas::new(info.pixel_w,info.pixel_h,BLACK,shape);
        let mut status=Status::Running;
        for tick in 0..=240 {
            if let Some(frame)=trace["frames"].as_array().unwrap().iter().find(|f|f["tick"]==tick) {
                let context=format!("seed={seed}, shape={shape_name}, tick={tick}");
                assert_eq!(g.score(),frame["score"].as_i64().unwrap(),"score: {context}");
                assert_eq!(status as i32,frame["status"].as_i64().unwrap() as i32,"status: {context}");
                assert_eq!(serde_json::to_value(g.hud()).unwrap(),frame["hud"],"HUD: {context}");
                canvas.clear();g.draw(&mut canvas);let mut hash=Sha256::new();
                for pixel in canvas.pixels(){hash.update(pixel.0.to_le_bytes());}
                assert_eq!(format!("{:x}",hash.finalize()),frame["pixels"].as_str().unwrap(),"pixels: {context}");
            }
            match tick {
                0=>g.handle_key(Key::Left),2=>g.handle_key_up(Key::Left),
                5|150=>g.handle_key(Key::Up),6|151=>g.handle_key_up(Key::Up),
                12|90=>g.handle_key(Key::A),13|91=>g.handle_key_up(Key::A),
                20=>g.handle_key(Key::Right),30=>g.handle_key_up(Key::Right),
                45=>g.handle_key(Key::Down),60=>g.handle_key_up(Key::Down),_=>{}
            }
            if status!=Status::GameOver {status=g.update();}
        }
    }
}
