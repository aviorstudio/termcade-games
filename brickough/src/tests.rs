use super::*;
#[test]
fn rebound_preserves_speed_and_clamps_to_sixty_degrees() {
    let center = bounce(30., 0.);
    assert!(center.x.abs() < 1e-9);
    assert!(center.y < 0.);
    for offset in [-1., -0.5, 0., 0.3, 1.] {
        let v = bounce(42., offset);
        assert!((v.len() - 42.).abs() < 1e-9);
        assert!(v.y < 0.);
    }
    let right = bounce(30., 1.);
    let left = bounce(30., -1.);
    assert!((right.x - 30. * (PI / 3.).sin()).abs() < 1e-9);
    assert!((left.x + right.x).abs() < 1e-9);
    assert_eq!(bounce(30., 5.), right);
}
use std::f64::consts::PI;
#[test]
fn collision_uses_shallow_axis_and_requires_positive_overlap() {
    let brick = Box2 {
        x: 10.,
        y: 10.,
        w: 8.,
        h: 3.,
    };
    assert!(
        hit(
            Box2 {
                x: 9.5,
                y: 10.5,
                w: 2.,
                h: 2.
            },
            brick
        ) == Axis::X
    );
    assert!(
        hit(
            Box2 {
                x: 13.,
                y: 8.5,
                w: 2.,
                h: 2.
            },
            brick
        ) == Axis::Y
    );
    for b in [
        Box2 {
            x: 0.,
            y: 0.,
            w: 2.,
            h: 2.,
        },
        Box2 {
            x: 8.,
            y: 10.,
            w: 2.,
            h: 2.,
        },
    ] {
        assert!(hit(b, brick) == Axis::None);
    }
}
#[test]
fn level_rows_cap_and_bricks_start_inside_field() {
    assert_eq!(level_bricks(1).len(), 32);
    assert_eq!(level_bricks(3).len(), 48);
    assert_eq!(level_bricks(9).len(), 48);
    for b in level_bricks(3) {
        assert!(b.alive && b.hp >= 1);
        assert!(
            b.rect.x >= 0.
                && b.rect.x + b.rect.w <= W
                && b.rect.y >= 0.
                && b.rect.y + b.rect.h <= PADDLE_Y
        );
    }
}
#[test]
fn high_speed_ball_does_not_tunnel_through_live_bricks() {
    let mut g = Brickough::new();
    g.reset();
    g.level = 9;
    g.start_level();
    g.stuck = false;
    g.vel = bounce(g.speed(), 0.7);
    g.ball = Vec2::new(32., 30.);
    for _ in 0..sdk::TPS * 60 {
        if g.update() == Status::GameOver {
            g.lives = 3;
            g.reset_ball();
            g.stuck = false;
            g.vel = bounce(g.speed(), -0.4);
            continue;
        }
        let ball = Box2 {
            x: g.ball.x - 1.,
            y: g.ball.y - 1.,
            w: 2.,
            h: 2.,
        };
        for b in &g.bricks {
            if b.alive && ball.overlaps(b.rect) {
                assert!(
                    !(g.ball.x > b.rect.x + 1.
                        && g.ball.x < b.rect.x + b.rect.w - 1.
                        && g.ball.y > b.rect.y + 1.
                        && g.ball.y < b.rect.y + b.rect.h - 1.)
                );
            }
        }
        if g.bricks.iter().all(|b| !b.alive) {
            break;
        }
    }
}
