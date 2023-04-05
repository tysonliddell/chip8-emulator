use fastrand::Rng;

use crate::memory::{
    CosmacRAM, INTERPRETER_WORK_AREA_START_ADDRESS, MEMORY_SIZE, PROGRAM_START_ADDRESS,
    STACK_START_ADDRESS,
};

pub struct Chip8Interpreter {
    rng: Rng,
}

// Program counter address
const PROGRAM_COUNTER_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS;
const I_ADDRESS: usize = INTERPRETER_WORK_AREA_START_ADDRESS + 2;

impl Chip8Interpreter {
    pub fn new() -> Self {
        Self { rng: Rng::new() }
    }

    pub fn reset(&self, ram: &mut CosmacRAM) {
        // reset all CHIP-8 interpreter state
        ram.zero_out_range(STACK_START_ADDRESS..MEMORY_SIZE)
            .expect("Should be ok to zero out this memory");

        ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, PROGRAM_START_ADDRESS as u16);
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
    /// -
    pub fn step(&self, ram: &mut CosmacRAM) {
        let instruction_address = ram.get_u16_at(PROGRAM_COUNTER_ADDRESS) as usize;
        let instruction = ram.get_u16_at(instruction_address);
        // let instruction = &ram.bytes()[instruction_address..instruction_address+2];
        // let (op1, op2) = (instruction[0], instruction[1]);

        let mut next_instruction_address = instruction_address.wrapping_add(2);

        match instruction {
            op if op & 0xF000 == 0x1000 => {
                // Unconditional jump
                let dest = op & 0x0FFF;
                next_instruction_address = dest as usize;
            }
            // op if op & 0xF000 == 0xB000 => {
            //     // Unconditional jump with offset
            //     let v0 = ram.get_v_registers()[0];
            //     let dest = (op & 0x0FFF).wrapping_add(v0 as u16);
            //     next_instruction_address = dest as usize;
            // }
            // op if op & 0xF000 == 0x2000 => {
            //     // Execute subroutine
            //     let dest = op & 0x0FFF;
            //     unimplemented!("Add subroutine jump")
            // }
            // op if op == 0x00EE => {
            //     // Return from subroutine
            //     unimplemented!("Add subroutine return")
            // }
            // op if op & 0xF000 == 0x3000 => {
            //     // Skip if VX == constant
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = ram.get_v_registers()[x as usize];
            //     let constant = (op & 0x00FF) as u8;
            //     if vx == constant {
            //         next_instruction_address = next_instruction_address.wrapping_add(2);
            //     }
            // }
            // op if op & 0xF000 == 0x4000 => {
            //     // Skip if VX != constant
            //     let x = (op & 0x0F00) >> 16;
            //     let vx = ram.get_v_registers()[x as usize];
            //     let constant = (op & 0x00FF) as u8;
            //     if vx != constant {
            //         next_instruction_address = next_instruction_address.wrapping_add(2);
            //     }
            // }
            // op if op & 0xF000 == 0x5000 => {
            //     // Skip if VX == VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vx = ram.get_v_registers()[x as usize];
            //     let vy = ram.get_v_registers()[y as usize];
            //     if vx == vy {
            //         next_instruction_address = next_instruction_address.wrapping_add(2);
            //     }
            // }
            // op if op & 0xF000 == 0x9000 => {
            //     // Skip if VX != VY
            //     let x = (op & 0x0F00) >> 16;
            //     let y = (op & 0x00F0) >> 8;
            //     let vx = ram.get_v_registers()[x as usize];
            //     let vy = ram.get_v_registers()[y as usize];
            //     if vx != vy {
            //         next_instruction_address = next_instruction_address.wrapping_add(2);
            //     }
            // }
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
            _ => unimplemented!(),
        };

        ram.set_u16_at(PROGRAM_COUNTER_ADDRESS, next_instruction_address as u16);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::PROGRAM_COUNTER_ADDRESS,
        memory::{CosmacRAM, PROGRAM_START_ADDRESS},
    };

    use super::Chip8Interpreter;

    #[test]
    fn unconditional_jump() {
        let mut ram = CosmacRAM::new();
        let chip8 = Chip8Interpreter::new();

        ram.load_chip8_program(&[0x12, 0x34])
            .expect("Should be ok to load this small program.");
        chip8.reset(&mut ram);

        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0200);
        chip8.step(&mut ram);
        assert_eq!(ram.get_u16_at(PROGRAM_COUNTER_ADDRESS), 0x0234);
    }
}
