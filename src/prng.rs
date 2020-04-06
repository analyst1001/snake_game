/// A very basic LCG to produce pseudo random numbers

use spin::Mutex;
use lazy_static::lazy_static;
use crate::system_time;

const TWO_POWER_32: u64 = 1<<32;

lazy_static! {
    pub static ref PRNG: Mutex<PseudoRandomNumberGenerator> = {
        let seed = system_time::get_system_time_seed();
        let mut prng = PseudoRandomNumberGenerator { seed: seed };
        Mutex::new(prng)
    };
}

#[derive(Copy, Debug, Clone)]
pub struct PseudoRandomNumberGenerator {
    seed: u64,
}

impl PseudoRandomNumberGenerator {

    /// Create a new PRNG
    pub fn new(seed: u64) -> Self {
        PseudoRandomNumberGenerator {
            seed: seed,
        }
    }

    /// Generate and return next pseudo random number
    // Uses implementation from https://en.wikipedia.org/wiki/Linear_congruential_generator
    pub fn next(&mut self) -> u64 {
        self.seed = (1664525 * self.seed + 1013904223) % TWO_POWER_32;
        self.seed
    }
}
