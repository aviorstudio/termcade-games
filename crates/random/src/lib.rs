//! Seeded compatibility with the games' original math/rand stream.
//! Algorithm: Mitchell/Reeds additive feedback; Go Authors' BSD notice is in
//! THIRD_PARTY_NOTICES.txt at the repository root. No Go runtime is used.
mod cooked;
use cooked::COOKED;
const LENGTH: usize = 607;
const MASK: u64 = (1u64 << 63) - 1;
const INT32_MAX: i64 = (1i64 << 31) - 1;
pub struct Random {
    tap: usize,
    feed: usize,
    state: [u64; LENGTH],
}
impl Random {
    pub fn new(seed: i64) -> Self {
        let mut seed = seed % INT32_MAX;
        if seed < 0 {
            seed += INT32_MAX;
        }
        if seed == 0 {
            seed = 89_482_311;
        }
        let mut result = Self {
            tap: 0,
            feed: LENGTH - 273,
            state: [0; LENGTH],
        };
        let mut x = seed;
        for i in -20..LENGTH as i32 {
            x = seed_step(x);
            if i >= 0 {
                let mut value = (x as u64) << 40;
                x = seed_step(x);
                value ^= (x as u64) << 20;
                x = seed_step(x);
                value ^= x as u64;
                value ^= COOKED[i as usize] as u64;
                result.state[i as usize] = value;
            }
        }
        result
    }
    pub fn from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self::new(nanos as i64)
    }
    pub fn u64(&mut self) -> u64 {
        self.tap = (self.tap + LENGTH - 1) % LENGTH;
        self.feed = (self.feed + LENGTH - 1) % LENGTH;
        let value = self.state[self.feed].wrapping_add(self.state[self.tap]);
        self.state[self.feed] = value;
        value
    }
    pub fn i63(&mut self) -> i64 {
        (self.u64() & MASK) as i64
    }
    pub fn f64(&mut self) -> f64 {
        loop {
            let value = self.i63() as f64 / (1u64 << 63) as f64;
            if value != 1.0 {
                return value;
            }
        }
    }
    pub fn below(&mut self, n: usize) -> usize {
        assert!(
            n > 0 && n <= i32::MAX as usize,
            "game random bound must fit positive i32"
        );
        let n = n as u32;
        let sample = |rng: &mut Self| (rng.i63() >> 32) as u32;
        if n.is_power_of_two() {
            return (sample(self) & (n - 1)) as usize;
        }
        let limit = i32::MAX as u32 - ((1u32 << 31) % n);
        let mut value = sample(self);
        while value > limit {
            value = sample(self);
        }
        (value % n) as usize
    }
    pub fn permutation(&mut self, n: usize) -> Vec<usize> {
        let mut result = vec![0; n];
        for i in 0..n {
            let j = self.below(i + 1);
            result[i] = result[j];
            result[j] = i;
        }
        result
    }
}
fn seed_step(value: i64) -> i64 {
    let high = value / 44_488;
    let low = value % 44_488;
    let next = 48_271 * low - 3_399 * high;
    if next < 0 { next + INT32_MAX } else { next }
}
