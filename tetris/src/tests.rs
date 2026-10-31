use super::*;
fn game() -> Tetris {
    let mut g = Tetris::new();
    g.reset();
    g
}
fn tick(g: &mut Tetris, n: i32) {
    for _ in 0..n {
        if g.update() == Status::GameOver {
            break;
        }
    }
}
fn full_row(b: &mut Board, y: i32) {
    for x in 0..BOARD_W {
        b.cells[(y * BOARD_W + x) as usize] = sdk::GREEN;
    }
}
#[test]
fn board_walls_floor_ceiling_and_compaction() {
    let mut b = Board::default();
    for (x, y, blocked) in [
        (-1, 5, true),
        (10, 5, true),
        (3, 20, true),
        (3, -1, false),
        (3, 5, false),
    ] {
        assert_eq!(b.blocked(x, y), blocked);
    }
    b.cells[16 * 10] = sdk::RED;
    b.cells[16 * 10 + 1] = sdk::RED;
    full_row(&mut b, 17);
    b.cells[18 * 10 + 7] = sdk::RED;
    full_row(&mut b, 19);
    assert_eq!(b.clear_lines(), 2);
    assert_eq!(b.at(7, 19), sdk::RED);
    assert_eq!(b.at(0, 18), sdk::RED);
    assert_eq!(b.at(1, 18), sdk::RED);
    assert!((0..10).all(|x| b.at(x, 17) == Color(0)));
    for y in 0..20 {
        full_row(&mut b, y);
    }
    assert_eq!(b.clear_lines(), 20);
    assert!(b.cells.iter().all(|&c| c == Color(0)));
    b.lock(Piece {
        kind: 2,
        y: -1,
        x: 3,
        rot: 0,
    });
    assert!((0..10).all(|x| b.at(x, 19) == Color(0)));
}
#[test]
fn drop_distance_and_srs_wall_kicks() {
    let mut b = Board::default();
    let o = Piece {
        kind: 1,
        x: 3,
        ..Piece::default()
    };
    assert_eq!(b.drop_distance(o), 18);
    full_row(&mut b, 19);
    assert_eq!(b.drop_distance(o), 17);
    let b = Board::default();
    let i = Piece {
        kind: 0,
        rot: 1,
        x: -2,
        y: 5,
    };
    assert!(b.fits(i));
    let rotated = b.rotate(i, 1).unwrap();
    assert!(b.fits(rotated));
    assert_ne!(rotated.x, i.x);
    assert_eq!(b.rotate(o, 1).unwrap().cells(), o.cells());
    let mut b = Board::default();
    for y in 0..20 {
        full_row(&mut b, y);
    }
    let t = Piece {
        kind: 2,
        x: 3,
        y: 5,
        rot: 0,
    };
    for [x, y] in t.cells() {
        b.cells[(y * 10 + x) as usize] = Color(0);
    }
    assert!(b.fits(t));
    assert!(b.rotate(t, 1).is_none());
}
#[test]
fn geometry_has_four_distinct_connected_squares_and_rotates_footprint() {
    for kind in 0..7 {
        assert_ne!(COLORS[kind], Color(0));
        let mut dims = Vec::new();
        for offsets in OFFSETS[kind] {
            let set: std::collections::BTreeSet<_> = offsets.into_iter().collect();
            assert_eq!(set.len(), 4);
            assert!(
                offsets
                    .iter()
                    .all(|&[x, y]| (0..4).contains(&x) && (0..4).contains(&y))
            );
            let mut reached = std::collections::BTreeSet::from([offsets[0]]);
            loop {
                let before = reached.len();
                for &p in &offsets {
                    if reached
                        .iter()
                        .any(|q: &[i32; 2]| (p[0] - q[0]).abs() + (p[1] - q[1]).abs() == 1)
                    {
                        reached.insert(p);
                    }
                }
                if before == reached.len() {
                    break;
                }
            }
            assert_eq!(reached.len(), 4);
            dims.push([
                offsets.iter().map(|p| p[0]).max().unwrap()
                    - offsets.iter().map(|p| p[0]).min().unwrap()
                    + 1,
                offsets.iter().map(|p| p[1]).max().unwrap()
                    - offsets.iter().map(|p| p[1]).min().unwrap()
                    + 1,
            ]);
        }
        for r in 0..4 {
            assert_eq!(dims[r], dims[(r + 2) & 3]);
            assert_eq!(dims[r], [dims[(r + 1) & 3][1], dims[(r + 1) & 3][0]]);
            if kind != 1 {
                let kicks = if kind == 0 {
                    KICKS_I[r][(r + 1) & 3]
                } else {
                    KICKS_JLSTZ[r][(r + 1) & 3]
                };
                assert_eq!(kicks[0], [0, 0]);
            }
        }
    }
    assert_eq!(
        Piece {
            kind: 2,
            x: 4,
            y: 6,
            rot: 0
        }
        .cells(),
        [[5, 6], [4, 7], [5, 7], [6, 7]]
    );
}
#[test]
fn seven_bag_deals_each_kind_once_per_round() {
    let mut bag = Bag::default();
    let mut rng = Random::new(7);
    for _ in 0..20 {
        let mut round = (0..7).map(|_| bag.take(&mut rng)).collect::<Vec<_>>();
        round.sort();
        assert_eq!(round, (0..7).collect::<Vec<_>>());
    }
}
#[test]
fn reset_gravity_and_hard_drop_preserve_score_and_debounce() {
    let mut g = game();
    assert!(!g.dead && g.board.fits(g.cur));
    assert_eq!((g.level, g.lines, g.score), (1, 0, 0));
    let y = g.cur.y;
    let interval = g.gravity();
    tick(&mut g, interval);
    assert_eq!(g.cur.y, y + 1);
    let slow = g.gravity();
    g.level = 10;
    assert!(g.gravity() < slow);
    g.level = 999;
    assert!(g.gravity() > 0);
    let mut g = game();
    let before = g.cur;
    let n = g.board.drop_distance(before);
    g.handle_key(Key::A);
    assert_eq!(g.score, 2 * i64::from(n));
    assert_eq!(g.cur.y, 0);
    assert!(
        before
            .cells()
            .iter()
            .any(|&[x, y]| g.board.at(x, y + n) != Color(0))
    );
    let score = g.score;
    g.handle_key(Key::A);
    assert_eq!(g.score, score);
    tick(&mut g, 12);
    g.handle_key(Key::A);
    assert!(g.score > score);
}
#[test]
fn line_scoring_levels_and_top_out() {
    let mut g = game();
    for y in 18..20 {
        full_row(&mut g.board, y);
        for x in [4, 5] {
            g.board.cells[(y * 10 + x) as usize] = Color(0);
        }
    }
    g.lines = 8;
    g.cur = Piece {
        kind: 1,
        x: 3,
        y: 18,
        rot: 0,
    };
    g.lock_piece();
    assert_eq!((g.lines, g.level, g.score), (10, 2, 300));
    let mut g = game();
    for y in 0..4 {
        full_row(&mut g.board, y);
    }
    g.spawn();
    assert!(g.dead);
    assert_eq!(g.update(), Status::GameOver);
}
#[test]
fn das_waits_then_repeats_and_walls_block_shifts() {
    let mut g = game();
    let x = g.cur.x;
    g.handle_key(Key::Left);
    g.update();
    assert_eq!(g.cur.x, x - 1);
    let held = g.cur.x;
    for _ in 0..8 {
        g.handle_key(Key::Left);
        g.update();
    }
    assert_eq!(g.cur.x, held);
    for _ in 0..6 {
        g.handle_key(Key::Left);
        g.update();
    }
    assert!(g.cur.x < held);
    for _ in 0..30 {
        g.shift(-1);
    }
    let p = g.cur;
    g.shift(-1);
    assert_eq!(g.cur, p);
    assert!(g.board.fits(p));
}
#[test]
fn grounded_lock_grace_expires_and_shuffle_resets_are_capped() {
    let mut g = game();
    g.cur = Piece {
        kind: 1,
        x: 3,
        y: 18,
        rot: 0,
    };
    tick(&mut g, 29);
    assert_eq!(g.board.at(4, 19), Color(0));
    tick(&mut g, 2);
    assert_ne!(g.board.at(4, 19), Color(0));
    let mut g = game();
    g.cur = Piece {
        kind: 1,
        x: 3,
        y: 18,
        rot: 0,
    };
    let mut peak = 0;
    for _ in 0..100 {
        g.shift(1);
        g.shift(-1);
        peak = peak.max(g.lock_resets);
        g.update();
        if g.board.at(4, 19) != Color(0) {
            break;
        }
    }
    assert_eq!(peak, 15);
    assert_ne!(g.board.at(4, 19), Color(0));
}
#[test]
fn scripted_play_keeps_scores_monotonic_and_live_pieces_legal() {
    let mut g = game();
    g.rng = Random::new(1);
    g.bag = Bag::default();
    let mut rng = Random::new(99);
    let mut last = 0;
    let mut c = Canvas::new(32, 40, sdk::BLACK, sdk::CellShape::quadrant());
    for _ in 0..20_000 {
        let key = match rng.below(20) {
            0 => Some(Key::Left),
            1 => Some(Key::Right),
            2 => Some(Key::Up),
            3 => Some(Key::Down),
            4 => Some(Key::A),
            _ => None,
        };
        if let Some(key) = key {
            g.handle_key(key);
        }
        let status = g.update();
        c.clear();
        g.draw(&mut c);
        let _ = c.render();
        assert!(g.score >= last);
        last = g.score;
        assert_eq!(g.level, g.lines / 10 + 1);
        if status == Status::GameOver {
            return;
        }
        assert!(g.board.fits(g.cur));
    }
}
