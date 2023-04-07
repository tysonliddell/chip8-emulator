use std::fmt::{self, Debug};

use fastrand::Rng;

use crate::{
    debug::{
        panic_if_chip8_stack_empty_on_subroutine_return, panic_if_chip8_stack_full,
        panic_if_pc_address_not_in_chip8_program_range,
    },
    memory::{
        CosmacRAM, INTERPRETER_WORK_AREA_START_ADDRESS, MEMORY_SIZE, PROGRAM_LAST_ADDRESS,
        PROGRAM_START_ADDRESS, STACK_START_ADDRESS,
    },
};

pub struct Chip8State<'a> {
    pub program_counter: u16,
    pub instruction: u16,
    pub i: u16,
    pub stack_pointer: u16,
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

pub struct Chip8Interpreter {
    rng: Rng,
}

// Program counter address
pub(crate) const PROGRAM_COUNTER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS;
pub(crate) const I_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 2;
pub(crate) const STACK_POINTER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 4;

impl Chip8Interpreter {
    pub fn new() -> Self {
        Self { rng: Rng::new() }
    }

    pub fn reset(&self, ram: &mut CosmacRAM) {
        // reset all CHIP-8 interpreter state
        ram.zero_out_range(STACK_START_ADDRESS..MEMORY_SIZE)
            .expect("Should be ok to zero out this memory");

        ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, PROGRAM_START_ADDRESS as u16);
        ram.set_u16_at(STACK_POINTER_ADDRESS, STACK_START_ADDRESS as u16);
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
            // op if op & 0xF0FF == 0xE09E => {
            //     // Skip if VX == Hex key (LSB)
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = ram.get_v_registers()[x as usize];
            //     let vx_lsb = vx & 0x0F;
            //     let key: Option<u8> = todo!("Grab the key currently being pressed.");
            //     if key.is_some() && key.unwrap() == vx_lsb  {
            //         next_instruction_address = next_instruction_address.wrapping_add(2);
            //     }
            // }
            // op if op & 0xF0FF == 0xE0A1 => {
            //     // Skip if VX != Hex key (LSB)
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = ram.get_v_registers()[x as usize];
            //     let vx_lsb = vx & 0x0F;
            //     let key: Option<u8> = todo!("Grab the key currently being pressed.");
            //     if key.is_none() || key.unwrap() != vx_lsb  {
            //         next_instruction_address = next_instruction_address.wrapping_add(2);
            //     }
            // }
            // op if op & 0xF000 == 0x6000 => {
            //     // Set VX = constant
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     let constant = (op & 0x00FF) as u8;
            //     *vx = constant;
            // }
            // op if op & 0xF000 == 0xC000 => {
            //     // Set VX = random bits.
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     let mask = (op & 0x00FF) as u8;
            //     let random_bits = self.rng.u8(..);
            //     *vx = mask & random_bits;
            // }
            // op if op & 0xF000 == 0x7000 => {
            //     // Set VX += constant
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     let constant = (op & 0x00FF) as u8;
            //     *vx = vx.wrapping_add(constant);
            // }
            // op if op & 0xF00F == 0x8000 => {
            //     // Set VX = VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vy_val = ram.get_v_registers()[y as usize];
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     *vx = vy_val;
            // }
            // op if op & 0xF00F == 0x8001 => {
            //     // Set VX = VX | VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vy_val = ram.get_v_registers()[y as usize];
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     *vx |= vy_val;
            // }
            // op if op & 0xF00F == 0x8002 => {
            //     // Set VX = VX & VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vy_val = ram.get_v_registers()[y as usize];
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     *vx &= vy_val;
            // }
            // op if op & 0xF00F == 0x8004 => {
            //     // Set VX = VX + VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vy_val = ram.get_v_registers()[y as usize];
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];

            //     let (sum, carry) = vx.overflowing_add(vy_val);
            //     *vx = sum;

            //     let vf = &mut ram.get_v_registers_mut()[0xF as usize];
            //     *vf = if carry { 1 } else { 0 };
            // }
            // op if op & 0xF00F == 0x8005 => {
            //     // Set VX = VX - VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vy_val = ram.get_v_registers()[y as usize];
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];

            //     let borrow = if *vx < vy_val { 0 } else { 1 };
            //     *vx = vx.wrapping_sub(vy_val);

            //     let vf = &mut ram.get_v_registers_mut()[0xF as usize];
            //     *vf = borrow;
            // }
            // op if op & 0xF0FF == 0xF007 => {
            //     // Set VX = timer
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     todo!("Implement timer logic");
            // }
            // op if op & 0xF0FF == 0xF00A => {
            //     // Set VX = hex key digit (wait for key press)
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = &mut ram.get_v_registers_mut()[x as usize];
            //     todo!("Implement hex key logic");
            // }
            // op if op & 0xF0FF == 0xF015 => {
            //     // Set timer = VX (01 = 1/60 seconds)
            //     let x = (op & 0x0F00) >> 16;
            //     let vx_val = ram.get_v_registers()[x as usize];
            //     todo!("Implement timer logic");
            // }
            // op if op & 0xF0FF == 0xF018 => {
            //     // Set tone duration = VX (01 = 1/60 seconds)
            //     let x = (op & 0x0F00) >> 16;
            //     let vx_val = ram.get_v_registers()[x as usize];
            //     todo!("Implement tone logic");
            // }
            // op if op & 0xF000 == 0xA000 => {
            //     // Set I = 0MMM
            //     let dest = op & 0x0FFF;
            //     ram.set_u16_at(I_ADDRESS, dest);
            // }
            // op if op & 0xF0FF == 0xF01E => {
            //     // Set I = I + VX
            //     let x = (op & 0x0F00) >> 16;
            //     let vx_val = ram.get_v_registers()[x as usize];

            //     let I_val = ram.get_u16_at(I_ADDRESS).wrapping_add(vx_val as u16);
            //     ram.set_u16_at(I_ADDRESS, I_val);
            // }
            // op if op & 0xF0FF == 0xF029 => {
            //     // Set I = Address of 5-byte display pattern for LSD of VX
            //     let x = (op & 0x0F00) >> 16;
            //     let vx_val = ram.get_v_registers()[x as usize];

            //     let I_val = todo!("Add logic for fonts");
            // }
            // op if op & 0xF0FF == 0xF033 => {
            //     // Set MI = 3-decimal digit equivalent of VX (I unchanged)
            //     let x = (op & 0x0F00) >> 16;
            //     let mut vx_val = ram.get_v_registers()[x as usize];

            //     todo!("See of we can make this more efficient. This seems slow.");
            //     let mut decimal_digits = [0u8; 3];
            //     decimal_digits[0] = vx_val / 100;
            //     vx_val -= decimal_digits[0];
            //     decimal_digits[1] = vx_val / 10;
            //     vx_val -= decimal_digits[1];
            //     decimal_digits[2] = vx_val;

            //     ram.load_bytes(&decimal_digits, I_ADDRESS).unwrap();
            // }
            _ => {
                #[cfg(debug_assertions)]
                dbg!(Self::get_state(ram));

                unimplemented!()
            }
        };

        #[cfg(debug_assertions)]
        panic_if_pc_address_not_in_chip8_program_range(next_instruction_address as u16);

        ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, next_instruction_address as u16);
    }

    pub fn get_state(ram: &CosmacRAM) -> Chip8State {
        let pc = ram.get_u16_at(PROGRAM_COUNTER_ADDRESS);

        Chip8State {
            program_counter: pc,
            instruction: ram.get_u16_at(pc as usize),
            i: ram.get_u16_at(I_ADDRESS),
            stack_pointer: ram.get_u16_at(STACK_POINTER_ADDRESS),
            v_registers: ram.get_v_registers(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use crate::{
        interpreter::PROGRAM_COUNTER_ADDRESS,
        memory::{CosmacRAM, PROGRAM_START_ADDRESS},
        test_utils,
    };

    use super::Chip8Interpreter;

    // Checks that a section of a CHIP-8 program steps through a sequence of
    // instruction addresses
    fn assert_address_sequence<I>(addresses: I, chip8: &Chip8Interpreter, ram: &mut CosmacRAM)
    where
        I: Iterator<Item = u16>,
    {
        for address in addresses {
            assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), address);
            chip8.step(ram);
        }
    }

    #[test]
    fn jump() {
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new();

        ram.load_chip8_program(&chip8_program_into_bytes!(0x1234))
            .expect("Should be ok to load this small program.");
        chip8.reset(&mut ram);

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0200);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0234);
    }

    #[test]
    fn jump_out_of_bounds() {
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new();

        ram.load_chip8_program(&chip8_program_into_bytes!(0x1234))
            .expect("Should be ok to load this small program.");
        chip8.reset(&mut ram);

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0200);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0234);
    }

    #[test]
    fn unconditional_jump_with_offset() {
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new();

        ram.load_chip8_program(&chip8_program_into_bytes!(0xB234))
            .expect("Should be ok to load this small program.");
        chip8.reset(&mut ram);

        let v0 = &mut ram.get_v_registers_mut()[0];
        *v0 = 0xAA;

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0200);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0234 + 0xAA);
    }

    #[test]
    fn subroutine() {
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new();

        let program = chip8_program_into_bytes!(
            0x2204  // 0x0200, jump to 0x0204 subroutine
            0x1208  // 0x0202, jump to end of program
            NOOP    // 0x0204
            0x00EE  // 0x0206, return from subroutine
            NOOP    // 0x0208
        );

        ram.load_chip8_program(&program)
            .expect("Should be ok to load this small program.");
        chip8.reset(&mut ram);

        let expected_address_sequence = [0x0200u16, 0x0204, 0x0206, 0x0202, 0x0208].into_iter();
        assert_address_sequence(expected_address_sequence, &chip8, &mut ram);
    }

    #[test]
    fn nested_subroutines() {
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new();

        // a program the dives into 12 nested subroutines then immediately
        // returns from each.
        let program = chip8_program_into_bytes!(
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
        );

        ram.load_chip8_program(&program)
            .expect("Should be ok to load this small program.");
        chip8.reset(&mut ram);

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
        let ram = &mut CosmacRAM::new();
        let chip8 = &Chip8Interpreter::new();

        let program = chip8_program_into_bytes!(
            0x3744  // 44 != 55, no skip expected
            0x3755  // 44 == 55, skip expected
            NOOP
            NOOP
        );
        ram.load_chip8_program(&program)
            .expect("Should be ok to load this small program.");
        chip8.reset(ram);
        ram.get_v_registers_mut()[7] = 0x55;

        let expected_address_sequence = [0x0200, 0x0202, 0x0206].into_iter();
        assert_address_sequence(expected_address_sequence, chip8, ram);
    }

    #[test]
    fn skip_instruction_if_vx_neq_kk() {
        let ram = &mut CosmacRAM::new();
        let chip8 = &Chip8Interpreter::new();

        let program = chip8_program_into_bytes!(
            0x4744  // 44 == 44, no skip expected
            0x4755  // 55 != 44, skip expected
            NOOP
            NOOP
        );
        ram.load_chip8_program(&program)
            .expect("Should be ok to load this small program.");
        chip8.reset(ram);
        ram.get_v_registers_mut()[7] = 0x44;

        let expected_address_sequence = [0x0200, 0x0202, 0x0206].into_iter();
        assert_address_sequence(expected_address_sequence, chip8, ram);
    }

    #[test]
    fn skip_instruction_if_vx_eq_vy() {
        let ram = &mut CosmacRAM::new();
        let chip8 = &Chip8Interpreter::new();

        let program = chip8_program_into_bytes!(
            0x5120
            NOOP
            NOOP
        );
        ram.load_chip8_program(&program)
            .expect("Should be ok to load this small program.");

        // V0 != V1
        chip8.reset(ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x22;

        chip8.step(ram);
        assert_eq!(0x0202, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));

        // V0 == V1
        chip8.reset(ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x11;

        chip8.step(ram);
        assert_eq!(0x0204, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));
    }

    #[test]
    fn skip_instruction_if_vx_neq_vy() {
        let ram = &mut CosmacRAM::new();
        let chip8 = &Chip8Interpreter::new();

        let program = chip8_program_into_bytes!(
            0x9120
            NOOP
            NOOP
        );
        ram.load_chip8_program(&program)
            .expect("Should be ok to load this small program.");

        // V0 == V1
        chip8.reset(ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x11;

        chip8.step(ram);
        assert_eq!(0x0202, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));

        // V0 != V1
        chip8.reset(ram);
        ram.get_v_registers_mut()[1] = 0x11;
        ram.get_v_registers_mut()[2] = 0x22;

        chip8.step(ram);
        assert_eq!(0x0204, ram.get_u16_at(PROGRAM_COUNTER_ADDRESS));
    }
}
