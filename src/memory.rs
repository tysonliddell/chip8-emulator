//! The CHIP-8 that ran on the COSMAC VIP had 2048 or 4096 bytes of memory,
//! divided into pages of 256 bytes each.
//!
//! # Memory map
//! The diagram below shows the layout where CAPACITY is 0x1000 or 0x0800.
//!
//! +-----------------------------------------+ 0x000
//! | CHIP-8 language interpreter (512 bytes) |
//! +-----------------------------------------+ 0x200
//! | User program/rom (1184 or 3232 bytes)   |
//! +-----------------------------------------+ CAPACITY - 256 - 48 - 48
//! | CHIP-8 stack (48 bytes)                 |
//! +-----------------------------------------+ CAPACITY - 256 - 48
//! | CHIP-8 interpreter work area (48 bytes) |
//! | Last 16 bytes contain V0-VF registers   |
//! +-----------------------------------------+ CAPACITY - 256
//! | Display refresh (256 bytes)             |
//! +-----------------------------------------+ CAPACITY
//! | Operating system ROM                    |
//! +-----------------------------------------+
//!
//! 4K memory map
//! +-----------------------------------------+ 0x0000
//! | CHIP-8 language interpreter (512 bytes) |
//! +-----------------------------------------+ 0x0200
//! | User program/rom (1184 or 3232 bytes)   |
//! +-----------------------------------------+ 0x0EA0
//! | CHIP-8 stack (48 bytes)                 |
//! +-----------------------------------------+ 0x0ED0
//! | CHIP-8 interpreter work area (48 bytes) |
//! | 0x0EF0 - 0x0EFF contain V0-VF registers |
//! +-----------------------------------------+ 0x0F00
//! | Display refresh (256 bytes)             |
//! +-----------------------------------------+ 0x1000
//! | Operating system ROM                    |
//! +-----------------------------------------+
//!
//! 2K memory map
//! +-----------------------------------------+ 0x0000
//! | CHIP-8 language interpreter (512 bytes) |
//! +-----------------------------------------+ 0x0200
//! | User program/rom (1184 or 3232 bytes)   |
//! +-----------------------------------------+ 0x06A0
//! | CHIP-8 stack (48 bytes)                 |
//! +-----------------------------------------+ 0x06D0
//! | CHIP-8 interpreter work area (48 bytes) |
//! | 0x06F0 - 0x06FF contain V0-VF registers |
//! +-----------------------------------------+ 0x0700
//! | Display refresh (256 bytes)             |
//! +-----------------------------------------+ 0x0800
//! | Operating system ROM                    |
//! +-----------------------------------------+
//!
//! # Running a CHIP-8 program on the COSMAC VIP
//! In normal operation, the COSMAC VIP natively runs CDP1802 machine language
//! instructions starting at address 0x0000. But first, to load a program into
//! RAM, the COSMAC operating system ROM is booted (by holding the `C` key on
//! the hex keyboard during startup). This allows each byte of the program to
//! be written to memory by hand, one byte at a time, using the hex keyboard or
//! read in from cassette tape.
//!
//! To run a CHIP-8 program on the COSMAC VIP, the CHIP-8 language interpreter,
//! written in CDP1802 machine language, first needs to be loaded into
//! addresses 0x0000 - 0x01FF. The CHIP-8 program itself then needs to be
//! loaded into memory starting at address 0x0200.
//!
//! The CHIP-8 stack is used by the CHIP-8 interpreter to store the subroutine
//! return addresses.
//!
//! The CHIP-8 interpreter work area contains the CHIP-8 "registers" and is used
//! by the interpreter (presumably to emulate the CHIP-8 fetch-decode-execute
//! cycle).
//!
//! The last page of RAM is used by the CHIP-8 interpreter for display refresh.
const _SMALL_MEMORY_SIZE: usize = 0x0800; // The 2K system
const _LARGE_MEMORY_SIZE: usize = 0x1000; // The beefier 4K system
pub const _MEMORY_SIZE: usize = _LARGE_MEMORY_SIZE;

pub const _MEMORY_START_ADDRESS: usize = 0x000;
pub const ROM_START_ADDRESS: usize = 0x200;
pub const STACK_START_ADDRESS: usize = 0xEA0;
pub const _INTERPRETER_START_ADDRESS: usize = 0x0ED0;
pub const _DISPLAY_REFRESH_START_ADDRESS: usize = 0xF00;

pub const ROM_LAST_ADDRESS: usize = STACK_START_ADDRESS - 1;

#[cfg(test)]
mod tests {
    use super::{
        ROM_LAST_ADDRESS, ROM_START_ADDRESS, STACK_START_ADDRESS, _DISPLAY_REFRESH_START_ADDRESS,
        _INTERPRETER_START_ADDRESS, _MEMORY_SIZE, _MEMORY_START_ADDRESS,
    };

    #[test]
    fn memory_boundaries() {
        assert_eq!(_MEMORY_SIZE, 4096);
        assert_eq!(_MEMORY_SIZE - _DISPLAY_REFRESH_START_ADDRESS, 256);
        assert_eq!(
            _DISPLAY_REFRESH_START_ADDRESS - _INTERPRETER_START_ADDRESS,
            48
        );
        assert_eq!(_INTERPRETER_START_ADDRESS - STACK_START_ADDRESS, 48);

        // ROMS get an extra 2048 bytes when using 4K of RAM instead of 2K.
        assert_eq!(STACK_START_ADDRESS - ROM_START_ADDRESS, 1184 + 2048);
        assert_eq!(ROM_LAST_ADDRESS, STACK_START_ADDRESS - 1);
        assert_eq!(ROM_START_ADDRESS - _MEMORY_START_ADDRESS, 512);
    }
}
