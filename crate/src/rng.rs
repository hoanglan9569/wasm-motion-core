//! xorshift32: the same petal rain on every reload.

pub struct Rng {
    pub state: u32,
}

impl Rng {
    pub const fn new(seed: u32) -> Self {
        Rng { state: seed }
    }

    pub fn seed(&mut self, seed: u32) {
        self.state = if seed == 0 { 0x9e37_79b9 } else { seed };
    }

    /// Uniform in [0, 1). State 0 means "never seeded": take the default.
    pub fn next(&mut self) -> f32 {
        if self.state == 0 {
            self.state = 0x9e37_79b9;
        }
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x >> 8) as f32 * (1.0 / 16_777_216.0)
    }
}
