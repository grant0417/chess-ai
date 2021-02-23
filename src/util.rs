pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub const fn new(seed: u64) -> Self {
        let mut s = seed;

        s += 0x9E3779B97f4A7C15;
        let mut result = s;
        result = (result ^ (result >> 30)) * 0xBF58476D1CE4E5B9;
        result = (result ^ (result >> 27)) * 0x94D049BB133111EB;
        let a = result ^ (result >> 31);

        s += 0x9E3779B97f4A7C15;
        let mut result = s;
        result = (result ^ (result >> 30)) * 0xBF58476D1CE4E5B9;
        result = (result ^ (result >> 27)) * 0x94D049BB133111EB;
        let b = result ^ (result >> 31);

        s += 0x9E3779B97f4A7C15;
        let mut result = s;
        result = (result ^ (result >> 30)) * 0xBF58476D1CE4E5B9;
        result = (result ^ (result >> 27)) * 0x94D049BB133111EB;
        let c = result ^ (result >> 31);

        s += 0x9E3779B97f4A7C15;
        let mut result = s;
        result = (result ^ (result >> 30)) * 0xBF58476D1CE4E5B9;
        result = (result ^ (result >> 27)) * 0x94D049BB133111EB;
        let d = result ^ (result >> 31);

        Rng { s: [a, b, c, d] }
    }

    pub fn unix_seed() -> Self {
        Self::new(std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos() as u64)
    }

    pub fn rand_u64(&mut self) -> u64 {
        let result = (self.s[1] * 5).rotate_left(7) * 9;
        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        return result;
    }

    pub const fn const_rand_u64(&self) -> (u64, Rng) {
        let result = (self.s[1] * 5).rotate_left(7) * 9;
        let t = self.s[1] << 17;

        let mut s0 = self.s[0];
        let mut s1 = self.s[1];
        let mut s2 = self.s[2];
        let mut s3 = self.s[3];

        s2 ^= s0;
        s3 ^= s1;
        s1 ^= s2;
        s0 ^= s3;

        s2 ^= t;
        s3 = s3.rotate_left(45);

        return (
            result,
            Rng {
                s: [s0, s1, s2, s3],
            },
        );
    }

    pub fn rand_f64(&mut self, low: f64, high: f64) -> f64 {
        low + (self.rand_u64()) as f64 / (u64::MAX as f64 / (high - low))
    }
}
