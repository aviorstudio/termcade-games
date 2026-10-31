mod geometry;
use geometry::{KICKS_I, KICKS_JLSTZ, OFFSETS};
use termcade_game_random::Random;
use termcade_sdk::{
    self as sdk, Canvas, Color, Game, Hud, HudField, Info, Key, KeyTracker, Status,
};

const BOARD_W: i32 = 10;
const BOARD_H: i32 = 20;
const GRAVITY: [i32; 20] = [
    48, 43, 38, 33, 28, 23, 18, 13, 8, 6, 5, 5, 5, 4, 4, 4, 3, 3, 3, 2,
];
const COLORS: [Color; 7] = [
    sdk::CYAN,
    sdk::YELLOW,
    sdk::PURPLE,
    sdk::GREEN,
    sdk::RED,
    sdk::BLUE,
    sdk::ORANGE,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Piece {
    kind: usize,
    rot: usize,
    x: i32,
    y: i32,
}
impl Piece {
    fn cells(self) -> [[i32; 2]; 4] {
        OFFSETS[self.kind][self.rot & 3].map(|[x, y]| [x + self.x, y + self.y])
    }
}
struct Board {
    cells: [Color; 200],
}
impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [Color(0); 200],
        }
    }
}
impl Board {
    fn at(&self, x: i32, y: i32) -> Color {
        if !(0..BOARD_W).contains(&x) || !(0..BOARD_H).contains(&y) {
            Color(0)
        } else {
            self.cells[(y * BOARD_W + x) as usize]
        }
    }
    fn blocked(&self, x: i32, y: i32) -> bool {
        !(0..BOARD_W).contains(&x) || y >= BOARD_H || (y >= 0 && self.at(x, y) != Color(0))
    }
    fn fits(&self, p: Piece) -> bool {
        p.cells().iter().all(|&[x, y]| !self.blocked(x, y))
    }
    fn lock(&mut self, p: Piece) {
        for [x, y] in p.cells() {
            if (0..BOARD_W).contains(&x) && (0..BOARD_H).contains(&y) {
                self.cells[(y * BOARD_W + x) as usize] = COLORS[p.kind];
            }
        }
    }
    fn clear_lines(&mut self) -> i32 {
        let mut dst = BOARD_H - 1;
        let mut cleared = 0;
        for src in (0..BOARD_H).rev() {
            if (0..BOARD_W).all(|x| self.at(x, src) != Color(0)) {
                cleared += 1;
                continue;
            }
            if src != dst {
                self.cells.copy_within(
                    (src * BOARD_W) as usize..((src + 1) * BOARD_W) as usize,
                    (dst * BOARD_W) as usize,
                );
            }
            dst -= 1;
        }
        if dst >= 0 {
            self.cells[..((dst + 1) * BOARD_W) as usize].fill(Color(0));
        }
        cleared
    }
    fn rotate(&self, p: Piece, dir: i32) -> Option<Piece> {
        let to = ((p.rot as i32 + dir) & 3) as usize;
        let kicks = if p.kind == 1 {
            [[0, 0]; 5]
        } else if p.kind == 0 {
            KICKS_I[p.rot][to]
        } else {
            KICKS_JLSTZ[p.rot][to]
        };
        kicks
            .into_iter()
            .map(|[x, y]| Piece {
                rot: to,
                x: p.x + x,
                y: p.y + y,
                ..p
            })
            .find(|&p| self.fits(p))
    }
    fn drop_distance(&self, mut p: Piece) -> i32 {
        let mut count = 0;
        loop {
            p.y += 1;
            if !self.fits(p) {
                return count;
            }
            count += 1;
        }
    }
}
#[derive(Default)]
struct Bag {
    queue: Vec<usize>,
}
impl Bag {
    fn take(&mut self, rng: &mut Random) -> usize {
        if self.queue.is_empty() {
            self.queue = rng.permutation(7);
        }
        self.queue.pop().expect("a fresh seven-bag is nonempty")
    }
}
pub struct Tetris {
    board: Board,
    cur: Piece,
    next: usize,
    bag: Bag,
    rng: Random,
    keys: KeyTracker,
    score: i64,
    lines: i32,
    level: i32,
    dead: bool,
    fall: i32,
    lock_timer: i32,
    lock_resets: i32,
    soft_tick: i32,
    das_left: i32,
    das_right: i32,
    rotate_cd: i32,
    drop_cd: i32,
}
impl Default for Tetris {
    fn default() -> Self {
        Self {
            board: Board::default(),
            cur: Piece::default(),
            next: 0,
            bag: Bag::default(),
            rng: Random::new(1),
            keys: KeyTracker::new(sdk::DEFAULT_HOLD_TICKS),
            score: 0,
            lines: 0,
            level: 0,
            dead: false,
            fall: 0,
            lock_timer: 0,
            lock_resets: 0,
            soft_tick: 0,
            das_left: 0,
            das_right: 0,
            rotate_cd: 0,
            drop_cd: 0,
        }
    }
}
impl Tetris {
    pub fn new() -> Self {
        Self::default()
    }
    fn spawn(&mut self) {
        self.cur = Piece {
            kind: self.next,
            x: 3,
            ..Piece::default()
        };
        self.next = self.bag.take(&mut self.rng);
        self.fall = 0;
        self.lock_timer = 0;
        self.lock_resets = 0;
        self.soft_tick = 0;
        if !self.board.fits(self.cur) {
            self.dead = true;
        }
    }
    fn grounded(&self) -> bool {
        !self.board.fits(Piece {
            y: self.cur.y + 1,
            ..self.cur
        })
    }
    fn touch_lock(&mut self) {
        if self.grounded() && self.lock_resets < 15 {
            self.lock_resets += 1;
            self.lock_timer = 0;
        }
    }
    fn shift(&mut self, dx: i32) {
        let p = Piece {
            x: self.cur.x + dx,
            ..self.cur
        };
        if self.board.fits(p) {
            self.cur = p;
            self.touch_lock();
        }
    }
    fn gravity(&self) -> i32 {
        GRAVITY[(self.level - 1).clamp(0, GRAVITY.len() as i32 - 1) as usize]
    }
    fn lock_piece(&mut self) {
        self.board.lock(self.cur);
        let n = self.board.clear_lines();
        if n > 0 {
            self.lines += n;
            self.score += [0, 100, 300, 500, 800][n.min(4) as usize] * i64::from(self.level);
            self.level = self.lines / 10 + 1;
        }
        self.spawn();
    }
    fn soft_drop(&mut self) {
        if !self.keys.held(Key::Down) {
            self.soft_tick = 0;
            return;
        }
        self.soft_tick += 1;
        if self.soft_tick < 2 {
            return;
        }
        self.soft_tick = 0;
        let p = Piece {
            y: self.cur.y + 1,
            ..self.cur
        };
        if self.board.fits(p) {
            self.cur = p;
            self.score += 1;
            self.fall = 0;
        }
    }
    fn apply_gravity(&mut self) {
        if self.grounded() {
            self.lock_timer += 1;
            if self.lock_timer >= 30 {
                self.lock_piece();
            }
            return;
        }
        self.lock_timer = 0;
        self.lock_resets = 0;
        self.fall += 1;
        if self.fall >= self.gravity() {
            self.fall = 0;
            self.cur.y += 1;
        }
    }
}
fn das(held: i32, down: bool) -> (i32, bool) {
    if !down {
        return (0, false);
    }
    let held = held + 1;
    (held, held == 1 || (held > 10 && (held - 10) % 2 == 0))
}
fn lighten(color: Color) -> Color {
    let (r, g, b) = color.rgb();
    let f = |v: u8| u32::from(v) + (255 - u32::from(v)) / 3;
    Color(f(r) << 16 | f(g) << 8 | f(b))
}
fn draw_block(c: &mut Canvas, x: f64, y: f64, color: Color) {
    c.fill_rect_f(x, y, 2., 2., color);
    c.fill_rect_f(x, y, 2., 1., lighten(color));
}
fn draw_preview(c: &mut Canvas, kind: usize) {
    let offsets = OFFSETS[kind][0];
    let min_x = offsets.iter().map(|p| p[0]).min().unwrap();
    let max_x = offsets.iter().map(|p| p[0]).max().unwrap();
    let min_y = offsets.iter().map(|p| p[1]).min().unwrap();
    let max_y = offsets.iter().map(|p| p[1]).max().unwrap();
    let ox = 23. + (f64::from(4 - (max_x - min_x + 1)) / 2. - f64::from(min_x)) * 2.;
    let oy = 6. + (f64::from(4 - (max_y - min_y + 1)) / 2. - f64::from(min_y)) * 2.;
    for [x, y] in offsets {
        draw_block(
            c,
            ox + f64::from(x) * 2.,
            oy + f64::from(y) * 2.,
            COLORS[kind],
        );
    }
}
impl Game for Tetris {
    fn info(&self) -> Info {
        Info {
            id: "aviorstudio/tetris".into(),
            title: "TETRIS".into(),
            pixel_w: 32,
            pixel_h: 40,
        }
    }
    fn reset(&mut self) {
        self.board = Board::default();
        self.score = 0;
        self.lines = 0;
        self.level = 1;
        self.dead = false;
        self.keys = KeyTracker::new(sdk::DEFAULT_HOLD_TICKS);
        self.rng = Random::from_clock();
        self.bag = Bag::default();
        self.next = self.bag.take(&mut self.rng);
        self.das_left = 0;
        self.das_right = 0;
        self.rotate_cd = 0;
        self.drop_cd = 0;
        self.spawn();
    }
    fn handle_key_up(&mut self, key: Key) {
        self.keys.release(key);
    }
    fn handle_key(&mut self, key: Key) {
        self.keys.press(key);
        if self.dead {
            return;
        }
        if key == Key::Up && self.rotate_cd == 0 {
            self.rotate_cd = 8;
            if let Some(p) = self.board.rotate(self.cur, 1) {
                self.cur = p;
                self.touch_lock();
            }
        }
        if key == Key::A && self.drop_cd == 0 {
            self.drop_cd = 12;
            let n = self.board.drop_distance(self.cur);
            self.cur.y += n;
            self.score += 2 * i64::from(n);
            self.lock_piece();
        }
    }
    fn update(&mut self) -> Status {
        self.keys.tick();
        if self.dead {
            return Status::GameOver;
        }
        if self.rotate_cd > 0 {
            self.rotate_cd -= 1;
        }
        if self.drop_cd > 0 {
            self.drop_cd -= 1;
        }
        let (held, moved) = das(self.das_left, self.keys.held(Key::Left));
        self.das_left = held;
        if moved {
            self.shift(-1);
        }
        let (held, moved) = das(self.das_right, self.keys.held(Key::Right));
        self.das_right = held;
        if moved {
            self.shift(1);
        }
        self.soft_drop();
        self.apply_gravity();
        if self.dead {
            Status::GameOver
        } else {
            Status::Running
        }
    }
    fn draw(&mut self, c: &mut Canvas) {
        for y in 0..BOARD_H {
            for x in 0..BOARD_W {
                let col = self.board.at(x, y);
                if col != Color(0) {
                    draw_block(c, f64::from(x) * 2., f64::from(y) * 2., col);
                }
            }
        }
        if !self.dead {
            let ghost = Piece {
                y: self.cur.y + self.board.drop_distance(self.cur),
                ..self.cur
            };
            for [x, y] in ghost.cells() {
                if y >= 0 {
                    c.fill_rect_f(f64::from(x) * 2., f64::from(y) * 2., 2., 2., sdk::DIM_GRAY);
                }
            }
            for [x, y] in self.cur.cells() {
                if y >= 0 {
                    draw_block(
                        c,
                        f64::from(x) * 2.,
                        f64::from(y) * 2.,
                        COLORS[self.cur.kind],
                    );
                }
            }
        }
        c.fill_rect_f(21., 0., 0.5, 40., sdk::DIM_GRAY);
        draw_preview(c, self.next);
    }
    fn score(&self) -> i64 {
        self.score
    }
    fn hud(&self) -> Hud {
        Hud {
            fields: vec![
                HudField {
                    label: "SCORE".into(),
                    value: format!("{:06}", self.score),
                    accent: true,
                },
                HudField {
                    label: "LEVEL".into(),
                    value: self.level.to_string(),
                    accent: false,
                },
                HudField {
                    label: "LINES".into(),
                    value: self.lines.to_string(),
                    accent: false,
                },
            ],
            hint: "←/→ move · ↑ rotate · ↓ soft · space drop".into(),
        }
    }
}
sdk::termcade_game!(Tetris::new);

#[cfg(test)]
mod trace_tests {
    use super::*;
    include!("../../internal/trace.rs");
    #[test]
    fn original_game_replays_match_all_four_densities() {
        replay(include_str!("../tests/fixtures/go-trace-v1.json"), |seed| {
            let mut g = Tetris::new();
            g.reset();
            g.rng = Random::new(seed);
            g.bag = Bag::default();
            g.next = g.bag.take(&mut g.rng);
            g.spawn();
            g
        });
    }
}

#[cfg(test)]
mod tests;
