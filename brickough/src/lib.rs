use termcade_sdk::{
    self as sdk, Canvas, Color, Game, Hud, HudField, Info, Key, KeyTracker, Status, Vec2,
};
const W: f64 = 64.;
const H: f64 = 40.;
const PADDLE_W: f64 = 12.;
const PADDLE_Y: f64 = 37.;
#[derive(Clone, Copy, Default)]
struct Box2 {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
impl Box2 {
    fn overlaps(self, b: Self) -> bool {
        self.x < b.x + b.w && self.x + self.w > b.x && self.y < b.y + b.h && self.y + self.h > b.y
    }
}
#[derive(Clone)]
struct Brick {
    rect: Box2,
    hp: i32,
    value: i64,
    color: Color,
    alive: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    None,
    X,
    Y,
}
fn bounce(speed: f64, offset: f64) -> Vec2 {
    let angle = offset.clamp(-1., 1.) * (60. * std::f64::consts::PI / 180.);
    Vec2::new(speed * angle.sin(), -speed * angle.cos())
}
fn hit(ball: Box2, brick: Box2) -> Axis {
    if !ball.overlaps(brick) {
        return Axis::None;
    }
    let x = (ball.x + ball.w).min(brick.x + brick.w) - ball.x.max(brick.x);
    let y = (ball.y + ball.h).min(brick.y + brick.h) - ball.y.max(brick.y);
    if x < y { Axis::X } else { Axis::Y }
}
fn level_bricks(level: i32) -> Vec<Brick> {
    let palette = [
        (2, 70, sdk::RED),
        (2, 60, sdk::ORANGE),
        (1, 50, sdk::YELLOW),
        (1, 40, sdk::GREEN),
        (1, 30, sdk::CYAN),
        (1, 20, sdk::BLUE),
    ];
    let rows = (3 + level).min(6) as usize;
    let mut bricks = Vec::with_capacity(rows * 8);
    for (r, &(hp, value, color)) in palette[6 - rows..].iter().enumerate() {
        for c in 0..8 {
            bricks.push(Brick {
                rect: Box2 {
                    x: c as f64 * 8.,
                    y: 4. + r as f64 * 3.,
                    w: 8.,
                    h: 3.,
                },
                hp,
                value,
                color,
                alive: true,
            });
        }
    }
    bricks
}
pub struct Brickough {
    paddle_x: f64,
    ball: Vec2,
    vel: Vec2,
    stuck: bool,
    bricks: Vec<Brick>,
    level: i32,
    lives: i32,
    score: i64,
    keys: KeyTracker,
}
impl Default for Brickough {
    fn default() -> Self {
        Self {
            paddle_x: 0.,
            ball: Vec2::default(),
            vel: Vec2::default(),
            stuck: false,
            bricks: Vec::new(),
            level: 0,
            lives: 0,
            score: 0,
            keys: KeyTracker::new(sdk::DEFAULT_HOLD_TICKS),
        }
    }
}
impl Brickough {
    pub fn new() -> Self {
        Self::default()
    }
    fn start_level(&mut self) {
        self.bricks = level_bricks(self.level);
        self.reset_ball();
    }
    fn reset_ball(&mut self) {
        self.paddle_x = W / 2.;
        self.stuck = true;
    }
    fn speed(&self) -> f64 {
        32. * 1.1_f64.powf((self.level - 1) as f64)
    }
    fn collide(&mut self) -> bool {
        let mut changed = false;
        if self.ball.x < 1. {
            self.ball.x = 1.;
            self.vel.x = self.vel.x.abs();
            changed = true;
        } else if self.ball.x > W - 1. {
            self.ball.x = W - 1.;
            self.vel.x = -self.vel.x.abs();
            changed = true;
        }
        if self.ball.y < 1. {
            self.ball.y = 1.;
            self.vel.y = self.vel.y.abs();
            changed = true;
        }
        let ball = Box2 {
            x: self.ball.x - 1.,
            y: self.ball.y - 1.,
            w: 2.,
            h: 2.,
        };
        let paddle = Box2 {
            x: self.paddle_x - 6.,
            y: PADDLE_Y,
            w: 12.,
            h: 2.,
        };
        if self.vel.y > 0. && ball.overlaps(paddle) {
            let offset = (self.ball.x - self.paddle_x) / 6.;
            self.vel = bounce(self.speed(), offset);
            self.ball.y = PADDLE_Y - 1.;
            return true;
        }
        for brick in &mut self.bricks {
            if !brick.alive {
                continue;
            }
            let axis = hit(ball, brick.rect);
            if axis == Axis::None {
                continue;
            }
            if axis == Axis::X {
                self.vel.x = -self.vel.x;
            } else {
                self.vel.y = -self.vel.y;
            }
            brick.hp -= 1;
            if brick.hp <= 0 {
                brick.alive = false;
                self.score += brick.value * self.level as i64;
            }
            return true;
        }
        changed
    }
}
fn brighten(color: Color) -> Color {
    let (r, g, b) = color.rgb();
    let f = |value: u8| value as u32 + (255 - value as u32) / 2;
    Color(f(r) << 16 | f(g) << 8 | f(b))
}
impl Game for Brickough {
    fn info(&self) -> Info {
        Info {
            id: "aviorstudio/brickough".into(),
            title: "BRICKOUGH".into(),
            pixel_w: 64,
            pixel_h: 40,
        }
    }
    fn reset(&mut self) {
        self.score = 0;
        self.lives = 3;
        self.level = 1;
        self.keys = KeyTracker::new(sdk::DEFAULT_HOLD_TICKS);
        self.start_level();
    }
    fn handle_key(&mut self, key: Key) {
        self.keys.press(key);
        if key == Key::A && self.stuck {
            self.stuck = false;
            self.vel = bounce(self.speed(), 0.2);
        }
    }
    fn handle_key_up(&mut self, key: Key) {
        self.keys.release(key);
    }
    fn update(&mut self) -> Status {
        self.keys.tick();
        if self.keys.held(Key::Left) {
            self.paddle_x -= 55. * sdk::DT;
        }
        if self.keys.held(Key::Right) {
            self.paddle_x += 55. * sdk::DT;
        }
        self.paddle_x = self.paddle_x.clamp(PADDLE_W / 2., W - PADDLE_W / 2.);
        if self.stuck {
            self.ball = Vec2::new(self.paddle_x, PADDLE_Y - 2.);
            return Status::Running;
        }
        let movement = self.vel.scale(sdk::DT);
        let steps = movement.x.abs().max(movement.y.abs()).ceil().max(1.) as usize;
        let mut delta = movement.scale(1. / steps as f64);
        for _ in 0..steps {
            self.ball = self.ball + delta;
            if self.collide() {
                delta = self.vel.scale(sdk::DT / steps as f64);
            }
            if self.ball.y > H {
                self.lives -= 1;
                if self.lives <= 0 {
                    return Status::GameOver;
                }
                self.reset_ball();
                return Status::Running;
            }
        }
        if self.bricks.iter().all(|b| !b.alive) {
            self.level += 1;
            self.start_level();
        }
        Status::Running
    }
    fn draw(&mut self, c: &mut Canvas) {
        for brick in &self.bricks {
            if !brick.alive {
                continue;
            }
            let color = if brick.hp > 1 {
                brighten(brick.color)
            } else {
                brick.color
            };
            c.fill_rect_f(
                brick.rect.x,
                brick.rect.y,
                brick.rect.w - 1.,
                brick.rect.h - 1.,
                color,
            );
        }
        c.fill_rect_f(self.paddle_x - 6., PADDLE_Y, 12., 2., sdk::WHITE);
        c.fill_circle(self.ball.x, self.ball.y, 1., sdk::YELLOW);
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
                    label: String::new(),
                    value: "● "
                        .repeat(self.lives.max(0) as usize)
                        .trim_end_matches(' ')
                        .into(),
                    accent: true,
                },
            ],
            hint: if self.stuck {
                "space to launch".into()
            } else {
                String::new()
            },
        }
    }
}
sdk::termcade_game!(Brickough::new);

#[cfg(test)]
mod trace_tests {
    use super::*;
    include!("../../internal/trace.rs");
    #[test]
    fn original_game_replays_match_all_four_densities() {
        replay(include_str!("../tests/fixtures/go-trace-v1.json"), |seed| {
            let mut g = Brickough::new();
            g.reset();
            let _ = seed;
            g
        });
    }
}

#[cfg(test)]
mod tests;
