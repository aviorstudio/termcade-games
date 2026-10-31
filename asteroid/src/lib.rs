use std::f64::consts::PI;
use termcade_game_random::Random;
use termcade_sdk::{self as sdk, Canvas, Game, Hud, HudField, Info, Key, KeyTracker, Status, Vec2};
const W: f64 = 72.;
const H: f64 = 40.;
const CLASSES: [(f64, f64, i64); 3] = [(6., 8., 20), (3.5, 13., 50), (2., 20., 100)];
const SHIP: [Vec2; 4] = [
    Vec2::new(3.2, 0.),
    Vec2::new(-2.2, 2.),
    Vec2::new(-1.2, 0.),
    Vec2::new(-2.2, -2.),
];
const FLAME: [Vec2; 3] = [
    Vec2::new(-1.6, 1.),
    Vec2::new(-3.6, 0.),
    Vec2::new(-1.6, -1.),
];
#[derive(Clone)]
struct Rock {
    pos: Vec2,
    vel: Vec2,
    size: usize,
    shape: Vec<Vec2>,
    rot: f64,
    spin: f64,
}
#[derive(Clone)]
struct Bullet {
    pos: Vec2,
    vel: Vec2,
    ttl: i32,
}
pub struct Asteroid {
    ship_pos: Vec2,
    ship_vel: Vec2,
    angle: f64,
    bullets: Vec<Bullet>,
    rocks: Vec<Rock>,
    wave: i32,
    lives: i32,
    score: i64,
    invuln: i32,
    frame: i64,
    thrust_on: bool,
    keys: KeyTracker,
    rng: Random,
}
impl Default for Asteroid {
    fn default() -> Self {
        Self {
            ship_pos: Vec2::default(),
            ship_vel: Vec2::default(),
            angle: 0.,
            bullets: Vec::new(),
            rocks: Vec::new(),
            wave: 0,
            lives: 0,
            score: 0,
            invuln: 0,
            frame: 0,
            thrust_on: false,
            keys: KeyTracker::new(sdk::DEFAULT_HOLD_TICKS),
            rng: Random::new(1),
        }
    }
}
impl Asteroid {
    pub fn new() -> Self {
        Self::default()
    }
    fn respawn(&mut self) {
        self.ship_pos = Vec2::new(W / 2., H / 2.);
        self.ship_vel = Vec2::default();
        self.angle = -PI / 2.;
        self.invuln = 120;
        self.bullets.clear();
    }
    fn spawn_wave(&mut self) {
        self.rocks.clear();
        for _ in 0..3 + self.wave {
            let pos = self.edge_pos();
            let rock = self.new_rock(0, pos);
            self.rocks.push(rock);
        }
    }
    fn edge_pos(&mut self) -> Vec2 {
        loop {
            let p = Vec2::new(self.rng.f64() * W, self.rng.f64() * H);
            if distance(p, self.ship_pos, W, H) >= 15. {
                return p;
            }
        }
    }
    fn new_rock(&mut self, size: usize, pos: Vec2) -> Rock {
        let (radius, speed, _) = CLASSES[size];
        let dir = self.rng.f64() * 2. * PI;
        let speed = speed * (0.7 + self.rng.f64() * 0.6);
        let count = 8 + self.rng.below(3);
        let mut shape = Vec::with_capacity(count);
        for i in 0..count {
            let theta = i as f64 * 2. * PI / count as f64 + self.rng.f64() * 0.35;
            let r = radius * (0.75 + self.rng.f64() * 0.4);
            shape.push(Vec2::new(r * theta.cos(), r * theta.sin()));
        }
        Rock {
            pos,
            vel: Vec2::new(dir.cos(), dir.sin()).scale(speed),
            size,
            shape,
            rot: 0.,
            spin: (self.rng.f64() - 0.5) * 3.,
        }
    }
    fn split(&mut self, rock: &Rock) -> Vec<Rock> {
        if rock.size >= 2 {
            return Vec::new();
        }
        let mut out = Vec::new();
        for sign in [1., -1.] {
            let mut child = self.new_rock(rock.size + 1, rock.pos);
            let angle = sign * (0.7 + self.rng.f64() * 0.4);
            child.vel = rock.vel.rotate(angle).scale(1.3);
            out.push(child);
        }
        out
    }
}
fn wrap(p: Vec2, w: f64, h: f64) -> Vec2 {
    let mut x = p.x % w;
    if x < 0. {
        x += w;
    }
    let mut y = p.y % h;
    if y < 0. {
        y += h;
    }
    Vec2::new(x, y)
}
fn distance(a: Vec2, b: Vec2, w: f64, h: f64) -> f64 {
    let mut dx = (a.x - b.x).abs();
    if dx > w / 2. {
        dx = w - dx;
    }
    let mut dy = (a.y - b.y).abs();
    if dy > h / 2. {
        dy = h - dy;
    }
    dx.hypot(dy)
}
fn transform(model: &[Vec2], angle: f64, pos: Vec2) -> Vec<Vec2> {
    model.iter().map(|p| p.rotate(angle) + pos).collect()
}
impl Game for Asteroid {
    fn info(&self) -> Info {
        Info {
            id: "aviorstudio/asteroid".into(),
            title: "ASTEROID".into(),
            pixel_w: 72,
            pixel_h: 40,
        }
    }
    fn reset(&mut self) {
        self.score = 0;
        self.lives = 3;
        self.wave = 1;
        self.keys = KeyTracker::new(sdk::DEFAULT_HOLD_TICKS);
        self.rng = Random::from_clock();
        self.respawn();
        self.spawn_wave();
    }
    fn handle_key(&mut self, key: Key) {
        self.keys.press(key);
        if key == Key::A && self.bullets.len() < 4 {
            let nose = SHIP[0].rotate(self.angle) + self.ship_pos;
            let dir = Vec2::new(self.angle.cos(), self.angle.sin());
            self.bullets.push(Bullet {
                pos: nose,
                vel: self.ship_vel + dir.scale(40.),
                ttl: 72,
            });
        }
    }
    fn handle_key_up(&mut self, key: Key) {
        self.keys.release(key);
    }
    fn update(&mut self) -> Status {
        self.keys.tick();
        self.frame += 1;
        if self.invuln > 0 {
            self.invuln -= 1;
        }
        if self.keys.held(Key::Left) {
            self.angle -= 4. * sdk::DT;
        }
        if self.keys.held(Key::Right) {
            self.angle += 4. * sdk::DT;
        }
        self.thrust_on = self.keys.held(Key::Up);
        if self.thrust_on {
            self.ship_vel =
                self.ship_vel + Vec2::new(self.angle.cos(), self.angle.sin()).scale(30. * sdk::DT);
        }
        self.ship_vel = self.ship_vel.scale(1. - 0.4 * sdk::DT);
        let speed = self.ship_vel.len();
        if speed > 45. {
            self.ship_vel = self.ship_vel.scale(45. / speed);
        }
        self.ship_pos = wrap(self.ship_pos + self.ship_vel.scale(sdk::DT), W, H);
        self.bullets.retain_mut(|b| {
            b.ttl -= 1;
            if b.ttl <= 0 {
                return false;
            }
            b.pos = wrap(b.pos + b.vel.scale(sdk::DT), W, H);
            true
        });
        for rock in &mut self.rocks {
            rock.pos = wrap(rock.pos + rock.vel.scale(sdk::DT), W, H);
            rock.rot += rock.spin * sdk::DT;
        }
        let mut spawned = Vec::new();
        let mut bi = 0;
        while bi < self.bullets.len() {
            let collision = self.rocks.iter().position(|rock| {
                distance(self.bullets[bi].pos, rock.pos, W, H) < CLASSES[rock.size].0
            });
            if let Some(ri) = collision {
                let rock = self.rocks.remove(ri);
                self.score += CLASSES[rock.size].2;
                spawned.extend(self.split(&rock));
                self.bullets.remove(bi);
            } else {
                bi += 1;
            }
        }
        self.rocks.extend(spawned);
        if self.invuln == 0
            && self
                .rocks
                .iter()
                .any(|r| distance(self.ship_pos, r.pos, W, H) < CLASSES[r.size].0 + 2.)
        {
            self.lives -= 1;
            if self.lives <= 0 {
                return Status::GameOver;
            }
            self.respawn();
        }
        if self.rocks.is_empty() {
            self.wave += 1;
            self.spawn_wave();
            self.invuln = 60;
        }
        Status::Running
    }
    fn draw(&mut self, c: &mut Canvas) {
        for rock in &self.rocks {
            c.polyline(&transform(&rock.shape, rock.rot, rock.pos), true, sdk::GRAY);
        }
        for bullet in &self.bullets {
            c.fill_circle(bullet.pos.x, bullet.pos.y, 0.6, sdk::YELLOW);
        }
        if self.invuln > 0 && (self.frame / 4) % 2 == 1 {
            return;
        }
        c.polyline(
            &transform(&SHIP, self.angle, self.ship_pos),
            true,
            sdk::WHITE,
        );
        if self.thrust_on && (self.frame / 4) % 2 == 0 {
            c.polyline(
                &transform(&FLAME, self.angle, self.ship_pos),
                false,
                sdk::ORANGE,
            );
        }
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
                    label: "WAVE".into(),
                    value: self.wave.to_string(),
                    accent: false,
                },
                HudField {
                    label: String::new(),
                    value: "▲ "
                        .repeat(self.lives.max(0) as usize)
                        .trim_end_matches(' ')
                        .into(),
                    accent: true,
                },
            ],
            hint: "←/→ turn · ↑ thrust · space fire".into(),
        }
    }
}
sdk::termcade_game!(Asteroid::new);

#[cfg(test)]
mod trace_tests {
    use super::*;
    include!("../../internal/trace.rs");
    #[test]
    fn original_game_replays_match_all_four_densities() {
        replay(include_str!("../tests/fixtures/go-trace-v1.json"), |seed| {
            let mut g = Asteroid::new();
            g.reset();
            g.rng = Random::new(seed);
            g.respawn();
            g.spawn_wave();
            g
        });
    }
}

#[cfg(test)]
mod tests;
