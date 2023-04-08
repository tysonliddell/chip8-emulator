use crate::{
    interpreter::STACK_POINTER_ADDRESS,
    memory::{
        CosmacRAM, MEMORY_START_ADDRESS, PROGRAM_LAST_ADDRESS, PROGRAM_START_ADDRESS,
        STACK_START_ADDRESS,
    },
};

pub fn panic_if_pc_address_not_in_chip8_program_range(address: u16) {
    if !(PROGRAM_START_ADDRESS..=PROGRAM_LAST_ADDRESS).contains(&(address as usize)) {
        panic!(
            "Attempt to set program counter to address {address:#X} which is outside of \
            CHIP-8 program address range."
        );
    }
}

pub fn panic_if_i_address_out_of_bounds(address: u16) {
    // `I` register needs to be able to access character glyphs, which lie before
    // PROGRAM_START_ADDRESS.
    if !(MEMORY_START_ADDRESS..=PROGRAM_LAST_ADDRESS).contains(&(address as usize)) {
        panic!(
            "Attempt to set I address to {address:#X} which is outside of \
            normal operating range."
        );
    }
}

pub fn panic_if_chip8_stack_empty_on_subroutine_return(ram: &CosmacRAM) {
    let sp = ram.get_u16_at(STACK_POINTER_ADDRESS);
    if sp == STACK_START_ADDRESS as u16 {
        panic!(
            "Cannot return when not in a subroutine. \
            CHIP-8 subroutine stack is empty!"
        );
    }
}

pub fn panic_if_chip8_stack_full(ram: &CosmacRAM) {
    if ram.get_u16_at(STACK_POINTER_ADDRESS) == STACK_START_ADDRESS as u16 + 12 * 2 {
        panic!(
            "CHIP-8 stack overflow! \
            COSMAC VIP only allows 12 levels of subroutine nesting."
        );
    }
}
