// /// Convert a slice of `u16` to a vector of `u8` populated in COSMAC byte order.
// /// This is useful when moving an array of 16-bit CHIP-8 instruction literals
// /// (stored as `u16`s) into the 8-bit [`CosmacRAM`], which is big endian.
// pub fn cosmac_bytes_from_u16(data: &[u16]) -> Vec<u8> {
//     data.iter().copied().flat_map(u16::to_be_bytes).collect()
// }

/// Convert u16 CHIP-8 instructions to a Vec<u8> of bytes in big endian order.
macro_rules! chip8_program_into_bytes {
    ($($t:tt) *) => {{
        let instructions: Vec<u16> = vec![$( convert!($t)),*];
        instructions.into_iter().flat_map(|val| val.to_be_bytes()).collect::<Vec<u8>>()
    }};
}
macro_rules! convert {
    (NOOP) => {
        0x7000
    };
    ($x: literal) => {
        $x
    };
}
