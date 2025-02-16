//! Implementation of Chip-8 Interpreter based on spec:
//! http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

const RAM_SIZE: usize = 4096;
// The original implementation of the Chip-8 language used a 64x32-pixel monochrome display with this format:
const SCREEN_HEIGHT: usize = 64;
const SCREEN_WIDTH: usize = 32;

// 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;
// 16-key hexadecimal keypad with the following layout:
const NUM_KEYS: usize = 16;

// Most Chip-8 programs start at location 0x200
const START_ADDR: u16 = 0x200;

pub struct Chip8Emulator {
    v_registers: [u8; NUM_REGS],
    // There is also a 16-bit register called I.
    // This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    i_register: u16,
    // The program counter (PC) should be 16-bit, and is used to store the currently executing address.
    program_counter: u16,
    memory: [u8; RAM_SIZE],
    //The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    stack_pointer: usize,
    //  The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to when
    // finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    stack: [u16; STACK_SIZE],
    // Tracks what pixels are on/off
    display: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
    // Tracks which keys are pressed
    keyboard: [bool; NUM_KEYS],
    //  The delay timer is active whenever the delay timer register (DT) is non-zero.
    // This timer does nothing more than subtract 1 from the value of DT at a rate of 60Hz. When DT reaches 0, it deactivates.
    delay_timer: u8,
    //  The sound timer is active whenever the sound timer register (ST) is non-zero.
    // This timer also decrements at a rate of 60Hz, however, as long as ST's value is greater than zero, the Chip-8 buzzer will sound.
    // When ST reaches zero, the sound timer deactivates.
    sound_timer: u8,
}

impl Chip8Emulator {
    pub const fn new() -> Self {
        Self {
            v_registers: [0; 16],
            i_register: 0,
            memory: [0; 4096],
            // TODO: Update this to use `START_ADDR`
            program_counter: 0,
            stack: [0; 16],
            stack_pointer: 0,
            display: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            keyboard: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    const fn read_opcode(&self) -> u16 {
        let op_byte_1 = self.memory[self.program_counter as usize] as u16;
        let op_byte_2 = self.memory[(self.program_counter + 1) as usize] as u16;
        (op_byte_1 << 8) | op_byte_2
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.program_counter += 2;

            /*
            nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
            n or nibble - A 4-bit value, the lowest 4 bits of the instruction
            x - A 4-bit value, the lower 4 bits of the high byte of the instruction
            y - A 4-bit value, the upper 4 bits of the low byte of the instruction
            kk or byte - An 8-bit value, the lowest 8 bits of the instruction
            */

            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;

            let addr = opcode & 0x0FFF;
            let byte = (opcode & 0x00FF) as u8;
            match (c, x, y, d) {
                (0, 0, 0, 0) => {
                    break;
                }
                (0, 0, 0xE, 0) => self.cls(),
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x1, _, _, _) => self.jmp(addr),
                (0x2, _, _, _) => self.call(addr),
                (0x3, _, _, _) => self.skip(x, byte),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!("opcode {:04x} is not implemented", opcode),
            }
        }
    }

    fn skip(&mut self, x: u8, byte: u8) {
        //  3xkk - SE Vx, byte
        // Skip next instruction if Vx = kk.
        //The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
        todo!()
    }

    fn skip_not_eq(&mut self, x: u8, byte: u8) {
        // 4xkk - SNE Vx, byte
        // Skip next instruction if Vx != kk.
        // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
        todo!()
    }

    fn skip_eq_registers(&mut self, x: u8, y: u8) {
        // 5xy0 - SE Vx, Vy
        // Skip next instruction if Vx = Vy.

        // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
        todo!()
    }

    fn load_register(&mut self, x: u8, byte: u8) {
        // 6xkk - LD Vx, byte
        // Set Vx = kk.
        //The interpreter puts the value kk into register Vx.
        todo!()
    }

    fn add_to_register(&mut self, x: u8, byte: u8) {
        // 7xkk - ADD Vx, byte
        // Set Vx = Vx + kk.
        // Adds the value kk to the value of register Vx, then stores the result in Vx.
        todo!()
    }

    fn load(&mut self, x: u8, y: u8) {
        // 8xy0 - LD Vx, Vy
        // Set Vx = Vy.
        // Stores the value of register Vy in register Vx.
        todo!()
    }

    fn or(&mut self, x: u8, y: u8) {
        // 8xy1 - OR Vx, Vy
        // Set Vx = Vx OR Vy.
        // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
        // A bitwise OR compares the corrseponding bits from two values, and if either bit is 1,
        // then the same bit in the result is also 1. Otherwise, it is 0.
        todo!()
    }
    fn xor(&mut self, x: u8, y: u8) {
        //8xy3 - XOR Vx, Vy
        // Set Vx = Vx XOR Vy.
        // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
        todo!()
    }
    fn sub(&self, x: u8, y: u8) {
        //8xy5 - SUB Vx, Vy
        //Set Vx = Vx - Vy, set VF = NOT borrow.

        // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
        todo!()
    }

    fn shr(&mut self, x: u8, y: u8) {
        // 8xy6 - SHR Vx {, Vy}
        // Set Vx = Vx SHR 1.
        //If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
        todo!()
    }

    fn subn(&mut self, x: u8, y: u8) {
        //8xy7 - SUBN Vx, Vy
        //Set Vx = Vy - Vx, set VF = NOT borrow.
        //If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
        todo!()
    }

    fn shl(&mut self, x: u8, y: u8) {
        // 8xyE - SHL Vx {, Vy}
        // Set Vx = Vx SHL 1.
        //If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
        todo!()
    }

    fn sne(&mut self, x: u8, y: u8) {
        //9xy0 - SNE Vx, Vy
        //Skip next instruction if Vx != Vy.
        //The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
        todo!()
    }

    fn cls(&mut self) {
        // 00E0 - CLS
        // Clear the display.
        todo!()
    }

    fn jmp(&mut self, addr: u16) {
        // 1nnn - JP addr
        // Jump to location nnn.
        //  The interpreter sets the program counter to nnn.
        todo!()
    }

    fn call(&mut self, addr: u16) {
        // 2nnn - CALL addr
        // Call subroutine at nnn.
        // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
        assert!(self.stack_pointer < self.stack.len(), "Stack overflow");

        if let Ok(v) = u16::try_from(self.program_counter) {
            self.stack[self.stack_pointer] = v;
            self.stack_pointer += 1;
            self.program_counter = addr;
        } else {
            panic!("{} is bigger than u16 can hold", self.program_counter);
        }
    }

    fn ret(&mut self) {
        // 00EE - RET
        // Return from a subroutine.
        // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
        assert!(self.stack_pointer != 0, "Stack underflow");

        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer];
    }

    const fn add_xy(&mut self, x: u8, y: u8) {
        // 8xy4 - ADD Vx, Vy
        // Set Vx = Vx + Vy, set VF = carry.
        //The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
        let arg1 = self.v_registers[x as usize];
        let arg2 = self.v_registers[y as usize];
        let (val, overflow_detected) = arg1.overflowing_add(arg2);
        self.v_registers[x as usize] = val;
        if overflow_detected {
            self.v_registers[0xF] = 1;
        } else {
            self.v_registers[0xF] = 0;
        }
    }
    fn ld_I() {
        // Annn - LD I, addr
        // Set I = nnn.
        // The value of register I is set to nnn.
        todo!()
    }

    fn jump_from() {
        // Bnnn - JP V0, addr
        // Jump to location nnn + V0.
        // The program counter is set to nnn plus the value of V0.
        todo!()
    }

    fn rand() {
        // Cxkk - RND Vx, byte
        // Set Vx = random byte AND kk.
        // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
        todo!()
    }

    fn display() {
        // Dxyn - DRW Vx, Vy, nibble
        // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
        // The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
        todo!()
    }

    fn skip_if_key() {
        // Ex9E - SKP Vx
        // Skip next instruction if key with the value of Vx is pressed.
        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
        todo!()
    }

    fn skip_not_key() {
        // ExA1 - SKNP Vx
        // Skip next instruction if key with the value of Vx is not pressed.
        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
        todo!()
    }

    fn set_delay() {
        // Fx07 - LD Vx, DT
        // Set Vx = delay timer value.
        // The value of DT is placed into Vx.
        todo!()
    }

    fn wait_timer() {
        // Fx0A - LD Vx, K
        // Wait for a key press, store the value of the key in Vx.
        // All execution stops until a key is pressed, then the value of that key is stored in Vx.
        todo!()
    }

    fn set_timer() {
        // Fx15 - LD DT, Vx
        // Set delay timer = Vx.
        // DT is set equal to the value of Vx.
        todo!()
    }

    fn set_sound_timer() {
        // Fx18 - LD ST, Vx
        // Set sound timer = Vx.
        // ST is set equal to the value of Vx.
        todo!()
    }

    fn add_register() {
        // Fx1E - ADD I, Vx
        // Set I = I + Vx.
        // The values of I and Vx are added, and the results are stored in I.
        todo!()
    }

    fn fx29() {
        // Fx29 - LD F, Vx
        // Set I = location of sprite for digit Vx.
        // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
        todo!()
    }

    fn fx33() {
        // Fx33 - LD B, Vx
        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
        // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    }

    fn fx55() {
        // Fx55 - LD [I], Vx
        // Store registers V0 through Vx in memory starting at location I.
        // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    }

    fn fx65() {
        // Fx65 - LD Vx, [I]
        // Read registers V0 through Vx from memory starting at location I.
        // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut cpu = Chip8Emulator::new();
        cpu.v_registers[0] = 5;
        cpu.v_registers[1] = 10;

        let mem = &mut cpu.memory;
        mem[0x000] = 0x21;
        mem[0x001] = 0x00;
        mem[0x002] = 0x21;
        mem[0x003] = 0x00;
        mem[0x004] = 0x00;
        mem[0x005] = 0x00;

        mem[0x100] = 0x80;
        mem[0x101] = 0x14;
        mem[0x102] = 0x80;
        mem[0x103] = 0x14;
        mem[0x104] = 0x00;
        mem[0x105] = 0xEE;

        cpu.run();

        assert_eq!(cpu.v_registers[0], 45);
    }
}
