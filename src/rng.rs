#[cfg_attr(test, mockall::automock)]
pub trait Chip8Rng {
    fn random_u8(&self) -> u8;
}

impl Chip8Rng for fastrand::Rng {
    fn random_u8(&self) -> u8 {
        self.u8(0..=255)
    }
}
