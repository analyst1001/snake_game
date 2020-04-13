/* Very basic LCG to produce pseudo random numbers
/  borrowed from https://en.wikipedia.org/wiki/Linear_congruential_generator
*/

use crate::system_time;
use lazy_static::lazy_static;
use spin::Mutex;

const TWO_POWER_THIRTY_TWO: u64 = 1 << 32;

lazy_static! {
    /// Singleton PRNG object
    pub static ref PRNG: Mutex<PseudoRandomNumberGenerator> = {
        let seed = system_time::get_system_time_seed();
        let mut prng = PseudoRandomNumberGenerator::new(seed);
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
        PseudoRandomNumberGenerator { seed: seed }
    }

    /// Generate and return next pseudo random number
    pub fn next(&mut self) -> u64 {
        self.seed = (1664525 * self.seed + 1013904223) % TWO_POWER_THIRTY_TWO;
        self.seed
    }
}
