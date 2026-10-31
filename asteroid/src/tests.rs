use super::*;
#[test]
fn torus_wrapping_and_collisions_cross_both_seams() {
    for (input, output) in [
        (Vec2::new(10., 10.), Vec2::new(10., 10.)),
        (Vec2::new(-1., 10.), Vec2::new(71., 10.)),
        (Vec2::new(10., -0.5), Vec2::new(10., 39.5)),
        (Vec2::new(73., 10.), Vec2::new(1., 10.)),
        (Vec2::new(10., 41.), Vec2::new(10., 1.)),
        (Vec2::new(72., 40.), Vec2::default()),
        (Vec2::new(-72.5, 0.), Vec2::new(71.5, 0.)),
    ] {
        assert_eq!(wrap(input, W, H), output);
    }
    assert_eq!(distance(Vec2::new(1., 20.), Vec2::new(71., 20.), W, H), 2.);
    assert_eq!(distance(Vec2::new(30., 1.), Vec2::new(30., 39.), W, H), 2.);
    assert_eq!(distance(Vec2::new(10., 10.), Vec2::new(13., 14.), W, H), 5.);
}
#[test]
fn rock_fragments_inherit_position_speed_and_diverge() {
    let mut g = Asteroid::new();
    g.rng = Random::new(42);
    let mut parent = g.new_rock(0, Vec2::new(30., 20.));
    parent.vel = Vec2::new(10., 0.);
    let kids = g.split(&parent);
    assert_eq!(kids.len(), 2);
    for k in &kids {
        assert_eq!(k.size, 1);
        assert_eq!(k.pos, parent.pos);
        assert!((k.vel.len() - 13.).abs() < 1e-9);
    }
    assert!(kids[0].vel.y * kids[1].vel.y <= 0.);
    let small = g.split(&kids[0]);
    assert_eq!(small.len(), 2);
    assert_eq!(small[0].size, 2);
    assert!(g.split(&small[0]).is_empty());
}
#[test]
fn shapes_and_ship_rotation_preserve_geometry() {
    let pos = Vec2::new(20., 20.);
    let nose = SHIP[0].rotate(0.) + pos;
    assert!(nose.x > pos.x);
    assert!((nose.y - pos.y).abs() < 1e-9);
    let nose = SHIP[0].rotate(-PI / 2.) + pos;
    assert!(nose.y < pos.y);
    assert!((nose.x - pos.x).abs() < 1e-9);
    let mut g = Asteroid::new();
    g.rng = Random::new(7);
    for _ in 0..100 {
        let rock = g.new_rock(0, Vec2::default());
        assert!((8..=10).contains(&rock.shape.len()));
        for p in rock.shape {
            assert!((6. * 0.74..=6. * 1.16).contains(&p.len()));
        }
    }
}
#[test]
fn complete_wave_splits_scores_and_advances() {
    let mut g = Asteroid::new();
    g.reset();
    g.rng = Random::new(42);
    g.spawn_wave();
    g.invuln = 1 << 30;
    assert_eq!(g.rocks.len(), 4);
    for frame in 0..sdk::TPS * 120 {
        if g.wave != 1 {
            break;
        }
        if frame % 5 == 0
            && let Some(rock) = g.rocks.first()
        {
            g.angle = (rock.pos.y - g.ship_pos.y).atan2(rock.pos.x - g.ship_pos.x);
            g.handle_key(Key::A);
        }
        g.update();
    }
    assert_eq!(g.wave, 2);
    assert_eq!(g.score, 4 * 20 + 8 * 50 + 16 * 100);
    assert_eq!(g.rocks.len(), 5);
}
