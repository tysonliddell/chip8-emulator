use std::fmt::{self, Debug};

use crate::{
    debug::{
        panic_if_chip8_stack_empty_on_subroutine_return, panic_if_chip8_stack_full,
        panic_if_i_address_out_of_bounds, panic_if_pc_address_not_in_chip8_program_range,
    },
    font::{CHARACTER_BYTES, CHARACTER_MAP},
    memory::{
        CosmacRAM, INTERPRETER_WORK_AREA_START_ADDRESS, MEMORY_SIZE, MEMORY_START_ADDRESS,
        PROGRAM_LAST_ADDRESS, PROGRAM_START_ADDRESS, STACK_START_ADDRESS,
    },
    rng::Chip8Rng,
};

pub struct Chip8State<'a> {
    pub program_counter: u16,
    pub instruction: u16,
    pub i: u16,
    pub stack_pointer: u16,
    pub timer: u16,
    pub tone_timer: u16,
    pub hex_key_status: u16,
    pub v_registers: &'a [u8],
}

impl<'a> Debug for Chip8State<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chip8State")
            .field(
                "program_counter",
                &format!("0x{:0>4X}", self.program_counter),
            )
            .field("instruction", &format!("0x{:0>4X}", self.instruction))
            .field("i", &format!("0x{:0>4X}", self.i))
            .field("stack_pointer", &format!("0x{:0>4X}", self.stack_pointer))
            .field("TIMER", &format!("0x{:0>4X}", self.timer))
            .field("TONE TIMER", &format!("0x{:0>4X}", self.tone_timer))
            .field("HEX_KEY_STATUS", &format!("0x{:0>4X}", self.hex_key_status))
            .field("V0", &format!("0x{:0>4X}", self.v_registers[0]))
            .field("V1", &format!("0x{:0>4X}", self.v_registers[1]))
            .field("V2", &format!("0x{:0>4X}", self.v_registers[2]))
            .field("V3", &format!("0x{:0>4X}", self.v_registers[3]))
            .field("V4", &format!("0x{:0>4X}", self.v_registers[4]))
            .field("V5", &format!("0x{:0>4X}", self.v_registers[5]))
            .field("V6", &format!("0x{:0>4X}", self.v_registers[6]))
            .field("V7", &format!("0x{:0>4X}", self.v_registers[7]))
            .field("V8", &format!("0x{:0>4X}", self.v_registers[8]))
            .field("V9", &format!("0x{:0>4X}", self.v_registers[9]))
            .field("VA", &format!("0x{:0>4X}", self.v_registers[10]))
            .field("VB", &format!("0x{:0>4X}", self.v_registers[11]))
            .field("VC", &format!("0x{:0>4X}", self.v_registers[12]))
            .field("VD", &format!("0x{:0>4X}", self.v_registers[13]))
            .field("VE", &format!("0x{:0>4X}", self.v_registers[14]))
            .field("VF", &format!("0x{:0>4X}", self.v_registers[15]))
            .finish()
    }
}

// Program counter address
pub(crate) const CHARACTER_BYTES_ADDRESS: usize = 0x0000;
pub(crate) const CHARACTER_MAP_ADDRESS: usize = CHARACTER_BYTES_ADDRESS + CHARACTER_BYTES.len();
pub(crate) const PROGRAM_COUNTER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS;
pub(crate) const I_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 2;
pub(crate) const STACK_POINTER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 4;
pub(crate) const TIMER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 6;
pub(crate) const TONE_TIMER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 8;

pub(crate) const HEX_KEY_STATUS_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 10;
const HEX_KEY_WAIT_FLAG: u16 = 0x1000;
const HEX_KEY_SEEN_WHILE_WAITING_FLAG: u16 = 0x0100;
const HEX_KEY_DEPRESSED_FLAG: u16 = 0x0010;
const HEX_KEY_LAST_PRESSED_MASK: u16 = 0x000F;

pub struct Chip8Interpreter<T: Chip8Rng = fastrand::Rng> {
    rng: T,
}

impl<T: Chip8Rng> Chip8Interpreter<T> {
    pub fn new(rng: T) -> Self {
        Self { rng }
    }

    pub fn reset(&self, ram: &mut CosmacRAM) {
        // reset all CHIP-8 interpreter state
        ram.zero_out_range(STACK_START_ADDRESS..MEMORY_SIZE)
            .expect("Should be ok to zero out this memory");
        Chip8Interpreter::<T>::load_fonts(ram, MEMORY_START_ADDRESS as u16);

        ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, PROGRAM_START_ADDRESS as u16);
        ram.set_u16_at(STACK_POINTER_ADDRESS, STACK_START_ADDRESS as u16);
    }

    fn load_fonts(ram: &mut CosmacRAM, start_address: u16) {
        ram.load_bytes(&CHARACTER_BYTES, 0x0000)
            .expect("Should be ok to load font data data in low memory.");
        ram.load_bytes(&CHARACTER_MAP, CHARACTER_MAP_ADDRESS)
            .expect("Should be ok to load character map in low memory.");
    }

    /// Execute the current CHIP-8 instruction, determined by the internal
    /// CHIP-8 program counter, and advance the program counter to point to the
    /// next instruction to execute.
    ///
    /// # Errors
    /// TODO
    ///
    /// # Panics
    /// TODO
    ///
    /// # Bad programs
    /// - Out of bounds memory?
    /// - looping forever?
    pub fn step(&self, ram: &mut CosmacRAM) {
        #[cfg(debug_assertions)]
        dbg!(Self::get_state(ram));

        let instruction_address = ram.get_u16_at(PROGRAM_COUNTER_ADDRESS) as usize;
        let instruction = ram.get_u16_at(instruction_address);

        let timer = ram.get_u16_at(TIMER_ADDRESS);
        if timer > 0 {
            ram.set_u16_at(TIMER_ADDRESS, timer - 1);
        }

        let tone_timer = ram.get_u16_at(TONE_TIMER_ADDRESS);
        if tone_timer > 0 {
            ram.set_u16_at(TONE_TIMER_ADDRESS, tone_timer - 1);
        }

        let hex_key_status = ram.get_u16_at(HEX_KEY_STATUS_ADDRESS);
        if hex_key_status & HEX_KEY_WAIT_FLAG != 0 {
            // FX07 instruction
            // waiting for key press or release
            if hex_key_status & HEX_KEY_DEPRESSED_FLAG != 0 {
                // key currently pressed
                ram.set_u16_at(
                    HEX_KEY_STATUS_ADDRESS,
                    hex_key_status | HEX_KEY_SEEN_WHILE_WAITING_FLAG,
                );

                // update VX register for FX07 instruction.
                let x = (instruction & 0x0F00) >> 8;
                let hex_key_status = ram.get_u16_at(HEX_KEY_STATUS_ADDRESS);
                let key = hex_key_status & HEX_KEY_LAST_PRESSED_MASK;

                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx = key as u8;
            } else if hex_key_status & HEX_KEY_SEEN_WHILE_WAITING_FLAG != 0 {
                // seen key pressed and released following wait

                // reset flags
                ram.set_u16_at(
                    HEX_KEY_STATUS_ADDRESS,
                    hex_key_status & !(HEX_KEY_WAIT_FLAG | HEX_KEY_SEEN_WHILE_WAITING_FLAG),
                );

                // complete FX07 instruction
                let next_instruction_address = instruction_address.wrapping_add(2);
                ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, next_instruction_address as u16);
            }
            return;
        }

        let mut next_instruction_address = instruction_address.wrapping_add(2);

        match instruction {
            op if op == 0x7000 => {
                // NOOP
            }
            op if op & 0xF000 == 0x1000 => {
                // Unconditional jump
                let dest = op & 0x0FFF;
                next_instruction_address = dest as usize;
            }
            op if op & 0xF000 == 0xB000 => {
                // Unconditional jump with offset
                let v0 = ram.get_v_registers()[0];
                let dest = (op & 0x0FFF).wrapping_add(v0 as u16);
                next_instruction_address = dest as usize;
            }
            op if op & 0xF000 == 0x2000 => {
                // Execute subroutine
                #[cfg(debug_assertions)]
                panic_if_chip8_stack_full(ram);

                let dest_address = op & 0x0FFF;
                let caller_address = ram.get_u16_at(PROGRAM_COUNTER_ADDRESS);

                // Push where we are jumping from onto the stack
                let sp = ram.get_u16_at(STACK_POINTER_ADDRESS);
                ram.set_u16_at(sp as usize, caller_address);
                ram.set_u16_at(STACK_POINTER_ADDRESS, sp + 2);

                // Jump
                next_instruction_address = dest_address as usize;
            }
            op if op == 0x00EE => {
                // Return from subroutine
                #[cfg(debug_assertions)]
                panic_if_chip8_stack_empty_on_subroutine_return(ram);

                // Pop return address off stack
                let sp = ram.get_u16_at(STACK_POINTER_ADDRESS) - 2;
                ram.set_u16_at(STACK_POINTER_ADDRESS, sp);
                let caller_address = ram.get_u16_at(sp as usize);

                // Jump
                next_instruction_address = caller_address as usize + 2;
            }
            op if op & 0xF000 == 0x3000 => {
                // Skip if VX == constant
                let x = (op & 0x0F00) >> 8;
                let vx = ram.get_v_registers()[x as usize];
                let constant = (op & 0x00FF) as u8;
                if vx == constant {
                    next_instruction_address = next_instruction_address.wrapping_add(2);
                }
            }
            op if op & 0xF000 == 0x4000 => {
                // Skip if VX != constant
                let x = (op & 0x0F00) >> 8;
                let vx = ram.get_v_registers()[x as usize];
                let constant = (op & 0x00FF) as u8;
                if vx != constant {
                    next_instruction_address = next_instruction_address.wrapping_add(2);
                }
            }
            op if op & 0xF00F == 0x5000 => {
                // Skip if VX == VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;
                let vx = ram.get_v_registers()[x as usize];
                let vy = ram.get_v_registers()[y as usize];
                if vx == vy {
                    next_instruction_address = next_instruction_address.wrapping_add(2);
                }
            }
            op if op & 0xF00F == 0x9000 => {
                // Skip if VX != VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;
                let vx = ram.get_v_registers()[x as usize];
                let vy = ram.get_v_registers()[y as usize];
                if vx != vy {
                    next_instruction_address = next_instruction_address.wrapping_add(2);
                }
            }
            op if op & 0xF0FF == 0xE09E => {
                // Skip if VX == Hex key (LSB)
                let x = (op & 0x0F00) >> 8;
                let vx = ram.get_v_registers()[x as usize];
                let vx_lsb = vx & 0x0F;
                let key: Option<u8> = Self::get_current_key_press(ram);
                if key.is_some() && key.unwrap() == vx_lsb {
                    next_instruction_address = next_instruction_address.wrapping_add(2);
                }
            }
            op if op & 0xF0FF == 0xE0A1 => {
                // Skip if VX != Hex key (LSB)
                let x = (op & 0x0F00) >> 8;
                let vx = ram.get_v_registers()[x as usize];
                let vx_lsb = vx & 0x0F;
                let key: Option<u8> = Self::get_current_key_press(ram);
                if key.is_none() || key.unwrap() != vx_lsb {
                    next_instruction_address = next_instruction_address.wrapping_add(2);
                }
            }
            op if op & 0xF000 == 0x6000 => {
                // Set VX = constant
                let x = (op & 0x0F00) >> 8;
                let constant = (op & 0x00FF) as u8;

                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx = constant;
            }
            op if op & 0xF000 == 0xC000 => {
                // Set VX = random bits.
                let x = (op & 0x0F00) >> 8;
                let mask = (op & 0x00FF) as u8;

                let vx = &mut ram.get_v_registers_mut()[x as usize];
                let random_bits = self.rng.random_u8();
                *vx = mask & random_bits;
            }
            op if op & 0xF000 == 0x7000 => {
                // Set VX += constant
                let x = (op & 0x0F00) >> 8;
                let constant = (op & 0x00FF) as u8;

                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx = vx.wrapping_add(constant);
            }
            op if op & 0xF00F == 0x8000 => {
                // Set VX = VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;

                let vy_val = ram.get_v_registers()[y as usize];
                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx = vy_val;
            }
            op if op & 0xF00F == 0x8001 => {
                // Set VX = VX | VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;

                let vy_val = ram.get_v_registers()[y as usize];
                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx |= vy_val;
            }
            op if op & 0xF00F == 0x8002 => {
                // Set VX = VX & VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;

                let vy_val = ram.get_v_registers()[y as usize];
                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx &= vy_val;
            }
            op if op & 0xF00F == 0x8004 => {
                // Set VX = VX + VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;

                let vy_val = ram.get_v_registers()[y as usize];
                let vx = &mut ram.get_v_registers_mut()[x as usize];

                let (sum, carry) = vx.overflowing_add(vy_val);
                *vx = sum;

                let vf = &mut ram.get_v_registers_mut()[0xF];
                *vf = if carry { 1 } else { 0 };
            }
            op if op & 0xF00F == 0x8005 => {
                // Set VX = VX - VY
                let x = (op & 0x0F00) >> 8;
                let y = (op & 0x00F0) >> 4;

                let vy_val = ram.get_v_registers()[y as usize];
                let vx = &mut ram.get_v_registers_mut()[x as usize];

                let borrow = if *vx < vy_val { 0 } else { 1 };
                *vx = vx.wrapping_sub(vy_val);

                let vf = &mut ram.get_v_registers_mut()[0xF];
                *vf = borrow;
            }
            op if op & 0xF0FF == 0xF007 => {
                // Set VX = timer
                let x = (op & 0x0F00) >> 8;
                let timer = ram.get_u16_at(TIMER_ADDRESS);

                let vx = &mut ram.get_v_registers_mut()[x as usize];
                *vx = (timer & 0xFF) as u8;
            }
            op if op & 0xF0FF == 0xF00A => {
                // Set VX = hex key digit (wait for key press)
                let hex_key_status = ram.get_u16_at(HEX_KEY_STATUS_ADDRESS);
                ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, hex_key_status | HEX_KEY_WAIT_FLAG);

                // since program counter was advanced at the beginning of the function,
                // we need to put it back.
                next_instruction_address = instruction_address;
            }
            op if op & 0xF0FF == 0xF015 => {
                // Set timer = VX (01 = 1/60 seconds)
                let x = (op & 0x0F00) >> 8;
                let vx_val = ram.get_v_registers()[x as usize];
                ram.set_u16_at(TIMER_ADDRESS, vx_val as u16);
            }
            op if op & 0xF0FF == 0xF018 => {
                // Set tone duration = VX (01 = 1/60 seconds)
                let x = (op & 0x0F00) >> 8;
                let vx_val = ram.get_v_registers()[x as usize];
                ram.set_u16_at(TONE_TIMER_ADDRESS, vx_val as u16);
            }
            op if op & 0xF000 == 0xA000 => {
                // Set I = 0MMM
                let dest = op & 0x0FFF;
                ram.set_u16_at(I_ADDRESS, dest);
            }
            op if op & 0xF0FF == 0xF01E => {
                // Set I = I + VX
                let x = (op & 0x0F00) >> 8;
                let vx_val = ram.get_v_registers()[x as usize];

                let i_val = ram.get_u16_at(I_ADDRESS);
                ram.set_u16_at(I_ADDRESS, i_val.wrapping_add(vx_val as u16));
            }
            op if op & 0xF0FF == 0xF029 => {
                // Set I = Address of 5-byte display pattern for LSD of VX
                let x = (op & 0x0F00) >> 8;
                let vx_val = ram.get_v_registers()[x as usize];
                let hex_val = vx_val & 0x0F; // LSB of VX

                let hex_glyph_address = ram.bytes()[CHARACTER_MAP_ADDRESS + hex_val as usize];
                ram.set_u16_at(I_ADDRESS, hex_glyph_address as u16);
            }
            op if op & 0xF0FF == 0xF033 => {
                // Set MI = 3-decimal digit equivalent of VX (I unchanged)
                let x = (op & 0x0F00) >> 8;
                let mut vx_val = ram.get_v_registers()[x as usize];

                let mut decimal_digits = [0u8; 3];
                decimal_digits[0] = vx_val / 100;
                vx_val -= decimal_digits[0] * 100;
                decimal_digits[1] = vx_val / 10;
                vx_val -= decimal_digits[1] * 10;
                decimal_digits[2] = vx_val;

                let i_data = ram.get_u16_at(I_ADDRESS);
                ram.load_bytes(&decimal_digits, i_data as usize)
                    .expect("I register should point to valid memory location");
            }
            op if op & 0xF0FF == 0xF055 => {
                // Set MI = V0 : VX, I = I + X + 1
                let x = (op & 0x0F00) >> 8;
                let i = ram.get_u16_at(I_ADDRESS);

                for x in 0..=x as usize {
                    let vx_val = ram.get_v_registers()[x];
                    ram.load_bytes(&[vx_val], i as usize + x)
                        .expect("I register should point to valid memory location");
                }

                ram.set_u16_at(I_ADDRESS, i + x + 1);
            }
            op if op & 0xF0FF == 0xF065 => {
                // Set V0 : VX = MI, I = I + X + 1
                let x = (op & 0x0F00) >> 8;
                let i = ram.get_u16_at(I_ADDRESS);

                for x in 0..=x as usize {
                    let val = ram.bytes()[i as usize + x];
                    ram.get_v_registers_mut()[x] = val;
                }

                ram.set_u16_at(I_ADDRESS, i + x + 1);
            }
            _ => {
                #[cfg(debug_assertions)]
                dbg!(Self::get_state(ram));

                unimplemented!()
            }
        };

        #[cfg(debug_assertions)]
        {
            panic_if_pc_address_not_in_chip8_program_range(next_instruction_address as u16);
            panic_if_i_address_out_of_bounds(ram.get_u16_at(I_ADDRESS));
        }

        ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, next_instruction_address as u16);
    }

    pub fn get_state(ram: &CosmacRAM) -> Chip8State {
        let pc = ram.get_u16_at(PROGRAM_COUNTER_ADDRESS);

        Chip8State {
            program_counter: pc,
            instruction: ram.get_u16_at(pc as usize),
            i: ram.get_u16_at(I_ADDRESS),
            stack_pointer: ram.get_u16_at(STACK_POINTER_ADDRESS),
            timer: ram.get_u16_at(TIMER_ADDRESS),
            tone_timer: ram.get_u16_at(TONE_TIMER_ADDRESS),
            hex_key_status: ram.get_u16_at(HEX_KEY_STATUS_ADDRESS),
            v_registers: ram.get_v_registers(),
        }
    }

    fn get_current_key_press(ram: &CosmacRAM) -> Option<u8> {
        let hex_key_status = ram.get_u16_at(HEX_KEY_STATUS_ADDRESS);
        if HEX_KEY_DEPRESSED_FLAG & hex_key_status == 0 {
            None
        } else {
            Some((hex_key_status & HEX_KEY_LAST_PRESSED_MASK) as u8)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use crate::{
        interpreter::{
            HEX_KEY_DEPRESSED_FLAG, HEX_KEY_LAST_PRESSED_MASK, HEX_KEY_STATUS_ADDRESS, I_ADDRESS,
            PROGRAM_COUNTER_ADDRESS, TIMER_ADDRESS, TONE_TIMER_ADDRESS,
        },
        memory::{CosmacRAM, PROGRAM_START_ADDRESS},
        rng::MockChip8Rng,
        test_utils,
    };

    use super::Chip8Interpreter;

    // Checks that a section of a CHIP-8 program steps through a sequence of
    // instruction addresses
    fn assert_address_sequence<I>(
        addresses: I,
        chip8: &Chip8Interpreter<MockChip8Rng>,
        ram: &mut CosmacRAM,
    ) where
        I: Iterator<Item = u16>,
    {
        for address in addresses {
            assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), address);
            chip8.step(ram);
        }
    }

    // Get a new CHIP-8 interpreter and RAM, reset and loaded with the provided
    // CHIP-8 program.
    fn new_chip8_with_program(program: &[u8]) -> (CosmacRAM, Chip8Interpreter<MockChip8Rng>) {
        let rng = MockChip8Rng::new();
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new(rng);
        ram.load_chip8_program(&program)
            .expect("Should be ok to load this test program.");
        chip8.reset(&mut ram);
        (ram, chip8)
    }

    #[test]
    fn jump() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(0x1234));

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0200);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0234);
    }

    #[test]
    fn unconditional_jump_with_offset() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(0xB234));

        let v0 = &mut ram.get_v_registers_mut()[0];
        *v0 = 0xAA;

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0200);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0234 + 0xAA);
    }

    #[test]
    fn subroutine() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x2204  // 0x0200, jump to 0x0204 subroutine
            0x1208  // 0x0202, jump to end of program
            NOOP    // 0x0204
            0x00EE  // 0x0206, return from subroutine
            NOOP    // 0x0208
        ));

        let expected_address_sequence = [0x0200u16, 0x0204, 0x0206, 0x0202, 0x0208].into_iter();
        assert_address_sequence(expected_address_sequence, &chip8, &mut ram);
    }

    #[test]
    fn nested_subroutines() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            // a program the dives into 12 nested subroutines then immediately
            // returns from each.
            0x2204      // 0x0200
            0x1232      // 0x0202
            0x2208      // 0x0204
            0x00EE
            0x220C
            0x00EE
            0x2210
            0x00EE
            0x2214
            0x00EE
            0x2218
            0x00EE
            0x221C
            0x00EE
            0x2220
            0x00EE
            0x2224
            0x00EE
            0x2228
            0x00EE
            0x222C
            0x00EE
            0x2230
            0x00EE
            0x00EE
            NOOP        // 0x0232
        ));

        // build an iterator of the sequence of all instruction addresses
        // expected when running the program
        let expected_call_stack: Vec<u16> = (0..12).map(|i| 0x0200 + i * 4).collect();
        let last_caller = expected_call_stack.last().unwrap();

        let filling_the_stack = expected_call_stack.iter().copied();
        let top_of_stack = iter::once(last_caller + 4);
        let unwinding_the_stack = expected_call_stack
            .iter()
            .copied()
            .rev()
            .map(|addr| addr + 2);
        let final_jump = iter::once(0x0232);
        let expected_address_sequence = filling_the_stack
            .chain(top_of_stack)
            .chain(unwinding_the_stack)
            .chain(final_jump);

        assert_address_sequence(expected_address_sequence, &chip8, &mut ram);
    }

    #[test]
    fn skip_instruction_if_vx_eq_kk() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x3744  // 44 != 55, no skip expected
            0x3755  // 44 == 55, skip expected
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x55;

        let expected_address_sequence = [0x0200, 0x0202, 0x0206].into_iter();
        assert_address_sequence(expected_address_sequence, &chip8, &mut ram);
    }

    #[test]
    fn skip_instruction_if_vx_neq_kk() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x4744  // 44 == 44, no skip expected
            0x4755  // 55 != 44, skip expected
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x44;

        let expected_address_sequence = [0x0200, 0x0202, 0x0206].into_iter();
        assert_address_sequence(expected_address_sequence, &chip8, &mut ram);
    }

    #[test]
    fn skip_instruction_if_vx_eq_vy() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x5120
            NOOP
            NOOP
        ));

        // V0 != V1
        chip8.reset(&mut ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x22;

        chip8.step(&mut ram);
        assert_eq!(0x0202, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));

        // V0 == V1
        chip8.reset(&mut ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x11;

        chip8.step(&mut ram);
        assert_eq!(0x0204, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));
    }

    #[test]
    fn skip_instruction_if_vx_neq_vy() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x9120
            NOOP
            NOOP
        ));

        // V0 == V1
        chip8.reset(&mut ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x11;

        chip8.step(&mut ram);
        assert_eq!(0x0202, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));

        // V0 != V1
        chip8.reset(&mut ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x22;

        chip8.step(&mut ram);
        assert_eq!(0x0204, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));
    }

    #[test]
    fn skip_instruction_if_vx_eq_hex_key_depressed_and_eq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE79E
            NOOP
            NOOP
        ));
        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0012); // key 2 currently pressed

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0204);
    }

    #[test]
    fn skip_instruction_if_vx_eq_hex_key_depressed_and_neq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE79E
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0011); // key 1 currently pressed

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0202);
    }

    #[test]
    fn skip_instruction_if_vx_eq_hex_key_released_and_eq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE79E
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
                                             // no key depressed, but key 2 was last pressed
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0002);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0202);
    }

    #[test]
    fn skip_instruction_if_vx_eq_hex_key_released_and_neq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE79E
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
                                             // no key depressed, but key 1 was last pressed
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0001);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0202);
    }

    #[test]
    fn skip_instruction_if_vx_neq_hex_key_depressed_and_eq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE7A1
            NOOP
            NOOP
        ));
        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0012); // key 2 currently pressed

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0202);
    }

    #[test]
    fn skip_instruction_if_vx_neq_hex_key_depressed_and_neq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE7A1
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0011); // key 1 currently pressed

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0204);
    }

    #[test]
    fn skip_instruction_if_vx_neq_hex_key_released_and_eq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE7A1
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
                                             // no key depressed, but key 2 was last pressed
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0002);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0204);
    }

    #[test]
    fn skip_instruction_if_vx_neq_hex_key_released_and_neq() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xE7A1
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x42; // LSB is hex key 2
                                             // no key depressed, but key 1 was last pressed
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0001);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0204);
    }

    #[test]
    fn set_vx_register_constant() {
        let (mut ram, chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x6499
            NOOP
        ));

        assert_eq!(ram.get_v_registers()[4], 0x00);
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0x99);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_random() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xC4A5
            NOOP
        ));

        chip8.rng.expect_random_u8().return_const(0b0111_0111);

        // 0xC4A5 gives a bitmask  -> 1010_0101
        // random pattern from rng -> 0111_0111
        // expected result ---------> 0010_0101
        assert_eq!(ram.get_v_registers()[4], 0x00);
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0b0010_0101);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vx_add_kk() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x74A5
            NOOP
        ));

        ram.get_v_registers_mut()[4] = 0x07;
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[4], 0xA5 + 0x07);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vy() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x8620
            NOOP
        ));

        ram.get_v_registers_mut()[6] = 0x07;
        ram.get_v_registers_mut()[2] = 0x42;
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[6], 0x42);
        assert_eq!(ram.get_v_registers()[2], 0x42);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vx_bitwise_or_vy() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x8121
            NOOP
        ));

        ram.get_v_registers_mut()[1] = 0b0011_0101;
        ram.get_v_registers_mut()[2] = 0b0110_0110;
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[1], 0b0111_0111);
        assert_eq!(ram.get_v_registers()[2], 0b0110_0110);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vx_bitwise_and_vy() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x8122
            NOOP
        ));

        ram.get_v_registers_mut()[1] = 0b0011_0101;
        ram.get_v_registers_mut()[2] = 0b0110_0110;
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[1], 0b0010_0100);
        assert_eq!(ram.get_v_registers()[2], 0b0110_0110);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vx_add_vy_no_carry() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x8124
            NOOP
        ));

        ram.get_v_registers_mut()[0x1] = 0xF0;
        ram.get_v_registers_mut()[0x2] = 0x0F;
        ram.get_v_registers_mut()[0xF] = 0x55; // carry register
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[0x1], 0xFF);
        assert_eq!(ram.get_v_registers()[0x2], 0x0F);
        assert_eq!(ram.get_v_registers()[0xF], 0x00); // carry should be zero
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vx_add_vy_with_carry() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x8124
            NOOP
        ));

        ram.get_v_registers_mut()[0x1] = 0xFF;
        ram.get_v_registers_mut()[0x2] = 0x03;
        ram.get_v_registers_mut()[0xF] = 0x55; // carry register
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[0x1], 0x02);
        assert_eq!(ram.get_v_registers()[0x2], 0x03);
        assert_eq!(ram.get_v_registers()[0xF], 0x01); // carry should be one
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_vx_sub_vy() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0x8015
            0x8235
            0x8455
            NOOP
        ));

        // vx == vy
        ram.get_v_registers_mut()[0x0] = 0xF0;
        ram.get_v_registers_mut()[0x1] = 0xF0;

        // vx > vy
        ram.get_v_registers_mut()[0x2] = 0xF0;
        ram.get_v_registers_mut()[0x3] = 0x0F;

        // vx < vy
        ram.get_v_registers_mut()[0x4] = 0x0F;
        ram.get_v_registers_mut()[0x5] = 0xF0;

        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[0x0], 0x00);
        assert_eq!(ram.get_v_registers()[0x1], 0xF0);
        assert_eq!(ram.get_v_registers()[0xF], 0x01); // carry should be one

        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[0x2], 0xE1);
        assert_eq!(ram.get_v_registers()[0x3], 0x0F);
        assert_eq!(ram.get_v_registers()[0xF], 0x01); // carry should be one

        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[0x4], 0x1F);
        assert_eq!(ram.get_v_registers()[0x5], 0xF0);
        assert_eq!(ram.get_v_registers()[0xF], 0x00); // carry should be zero
    }

    #[test]
    fn set_vx_register_to_current_timer_value() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF407
            NOOP
        ));

        ram.set_u16_at(TIMER_ADDRESS, 0x77);
        ram.get_v_registers_mut()[4] = 0xFF;
        chip8.step(&mut ram);

        assert_eq!(ram.get_v_registers()[4], 0x77 - 1);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_vx_register_to_current_hex_digit() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF40A
            NOOP
        ));

        // last press was 9, no key currently pressed
        ram.set_u16_at(HEX_KEY_STATUS_ADDRESS, 0x0009);
        ram.get_v_registers_mut()[4] = 0xFF;

        // hex key not pressed yet, program counter doesn't move
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0xFF);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x200);

        // hex key not pressed yet, program counter doesn't move
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0xFF);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x200);

        // hex key not pressed yet, program counter doesn't move
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0xFF);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x200);

        // 3 key pressed
        let hex_key_status = ram.get_u16_at(HEX_KEY_STATUS_ADDRESS);
        ram.set_u16_at(
            HEX_KEY_STATUS_ADDRESS,
            hex_key_status & !HEX_KEY_LAST_PRESSED_MASK | HEX_KEY_DEPRESSED_FLAG | 0x03,
        );

        // key pressed, don't advance program counter yet!
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0x03);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x200);

        // key pressed, don't advance program counter yet!
        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0x03);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x200);

        // key released, program continues
        let hex_key_status = ram.get_u16_at(HEX_KEY_STATUS_ADDRESS);
        ram.set_u16_at(
            HEX_KEY_STATUS_ADDRESS,
            hex_key_status & !HEX_KEY_DEPRESSED_FLAG,
        );

        chip8.step(&mut ram);
        assert_eq!(ram.get_v_registers()[4], 0x03);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_timer_eq_vx_and_countdown() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF715
            NOOP
            NOOP
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x02;
        assert_eq!(ram.get_u16_at(TIMER_ADDRESS), 0x00);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TIMER_ADDRESS), 0x02);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TIMER_ADDRESS), 0x01);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x204);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TIMER_ADDRESS), 0x00);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x206);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TIMER_ADDRESS), 0x00);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x208);
    }

    #[test]
    fn set_tone_timer_eq_vx_and_countdown() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF718
            NOOP
            NOOP
            NOOP
            NOOP
        ));

        ram.get_v_registers_mut()[7] = 0x2;
        assert_eq!(ram.get_u16_at(TONE_TIMER_ADDRESS), 0x00);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TONE_TIMER_ADDRESS), 0x02);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TONE_TIMER_ADDRESS), 0x01);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x204);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TONE_TIMER_ADDRESS), 0x00);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x206);

        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(TONE_TIMER_ADDRESS), 0x00);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x208);
    }

    #[test]
    fn set_i_eq_const() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xA123
            NOOP
        ));

        assert_eq!(ram.get_u16_at(I_ADDRESS), 0x0000);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(I_ADDRESS), 0x0123);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_i_eq_i_add_vx() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF41E
            NOOP
        ));

        ram.set_u16_at(I_ADDRESS, 0x0123);
        ram.get_v_registers_mut()[4] = 0x45;
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(I_ADDRESS), 0x0123 + 0x45);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
    }

    #[test]
    fn set_i_eq_vx_lsd_display_pattern() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF729  // use V7
            NOOP
        ));

        assert_eq!(ram.get_u16_at(I_ADDRESS), 0x0000);
        ram.get_v_registers_mut()[7] = 0x45; // LSB == 5 means we expect glyph for hex 5.

        chip8.step(&mut ram);

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);
        let hex_5_address = ram.get_u16_at(I_ADDRESS) as usize;
        let glyph = &ram.bytes()[hex_5_address..][..5];
        #[rustfmt::skip]
        assert_eq!(glyph, &[
            0b11110000,
            0b10000000,
            0b11110000,
            0b00010000,
            0b11110000,
        ]);
    }

    #[test]
    fn set_i_data_to_decimal_digits_of_vx() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xF133
            0xF233
            0xF333
            0xF433
            NOOP
        ));

        ram.get_v_registers_mut()[1] = 234; // 3 digit test case
        ram.get_v_registers_mut()[2] = 56; // 2 digit test case
        ram.get_v_registers_mut()[3] = 7; // 1 digit test case
        ram.get_v_registers_mut()[4] = 0; // zero test case
        ram.set_u16_at(I_ADDRESS, 0x0300); // write digits to memory address 0x0300

        chip8.step(&mut ram);
        let result = &ram.bytes()[0x0300..][..3];
        assert_eq!(result, &[2, 3, 4]);
        assert_eq!(
            ram.get_u16_at(I_ADDRESS),
            0x0300,
            "I register should be unchanged"
        );

        chip8.step(&mut ram);
        let result = &ram.bytes()[0x0300..][..3];
        assert_eq!(result, &[0, 5, 6]);
        assert_eq!(
            ram.get_u16_at(I_ADDRESS),
            0x0300,
            "I register should be unchanged"
        );

        chip8.step(&mut ram);
        let result = &ram.bytes()[0x0300..][..3];
        assert_eq!(result, &[0, 0, 7]);
        assert_eq!(
            ram.get_u16_at(I_ADDRESS),
            0x0300,
            "I register should be unchanged"
        );

        chip8.step(&mut ram);
        let result = &ram.bytes()[0x0300..][..3];
        assert_eq!(result, &[0, 0, 0]);
        assert_eq!(
            ram.get_u16_at(I_ADDRESS),
            0x0300,
            "I register should be unchanged"
        );
    }

    #[test]
    fn set_i_data_to_vx_slice() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xFC55
            NOOP
        ));

        // set each VX register to its index to generate some test data
        let test_register_vals = [
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF,
        ];
        ram.get_v_registers_mut()
            .copy_from_slice(&test_register_vals);

        // use I = 0x0300 and set some data at this location before executing the instruction
        ram.set_u16_at(I_ADDRESS, 0x0300);
        ram.load_bytes(&[0xFF; 16], 0x0300).unwrap();

        dbg!(&ram.bytes()[0x0300..][..16]);

        // execute the instruction
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);

        // data pointed to by I should be updated
        assert_eq!(
            &ram.bytes()[0x0300..][..16],
            &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xFF, 0xFF, 0xFF]
        );

        // value of I should be incremented on COSMAC VIP CHIP-8.
        assert_eq!(ram.get_u16_at(I_ADDRESS), 0x0300 + 0xC + 1);
    }

    #[test]
    fn set_vx_slice_to_i_data() {
        let (mut ram, mut chip8) = new_chip8_with_program(&chip8_program_into_bytes!(
            0xFC65
            NOOP
        ));

        // set I data
        ram.set_u16_at(I_ADDRESS, 0x0300);
        let test_data = [
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF,
        ];
        ram.load_bytes(&test_data, 0x300);

        // Fill VX registers with existing data
        ram.get_v_registers_mut().copy_from_slice(&[0xFF; 16]);

        // execute the instruction
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x202);

        // check data copied
        assert_eq!(
            ram.get_v_registers(),
            &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xFF, 0xFF, 0xFF]
        );

        // check I incremented
        assert_eq!(ram.get_u16_at(I_ADDRESS), 0x0300 + 0xC + 1);
    }
}
