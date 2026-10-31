use termcade_game_random::Random;
#[test]
fn original_go_seed_streams_floats_bounded_samples_and_bags_match() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/go-random-v1.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let seed = case["seed"].as_i64().unwrap();
        let mut rng = Random::new(seed);
        for expected in case["u64"].as_array().unwrap() {
            assert_eq!(rng.u64(), expected.as_u64().unwrap());
        }
        let mut rng = Random::new(seed);
        for expected in case["f64_bits"].as_array().unwrap() {
            assert_eq!(rng.f64().to_bits(), expected.as_u64().unwrap());
        }
        let mut rng = Random::new(seed);
        for expected in case["below3"].as_array().unwrap() {
            assert_eq!(rng.below(3), expected.as_u64().unwrap() as usize);
        }
        let mut rng = Random::new(seed);
        for expected in case["permutations7"].as_array().unwrap() {
            assert_eq!(
                rng.permutation(7),
                expected
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect::<Vec<_>>()
            );
        }
    }
}
