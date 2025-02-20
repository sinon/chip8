//! Implementation of Chip-8 Interpreter based on spec:
//! <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM>

#![no_std]

const RAM_SIZE: usize = 4096;
// The original implementation of the Chip-8 language used a 64x32-pixel monochrome display with this format:
pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;

// 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;
// 16-key hexadecimal keypad with the following layout:
const NUM_KEYS: usize = 16;

// Most Chip-8 programs start at location 0x200
const START_ADDR: u16 = 0x200;

const OPCODE_SIZE: u16 = 2;

// Programs may also refer to a group of sprites representing the hexadecimal digits 0 through F.
// These sprites are 5 bytes long, or 8x5 pixels. The data should be stored in the interpreter area of Chip-8 memory (0x000 to 0x1FF)
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.4
const FONT_SPRITES: [u8; 16 * 5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug)]
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

impl Default for Chip8Emulator {
    fn default() -> Self {
        Self {
            v_registers: Default::default(),
            i_register: Default::default(),
            program_counter: START_ADDR,
            memory: [0; 4096],
            stack_pointer: Default::default(),
            stack: Default::default(),
            display: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            keyboard: Default::default(),
            delay_timer: Default::default(),
            sound_timer: Default::default(),
        }
    }
}

impl Chip8Emulator {
    #[must_use]
    pub fn new() -> Self {
        let mut emu: Self = Default::default();
        emu.memory[..(16 * 5)].copy_from_slice(&FONT_SPRITES);
        emu
    }

    pub fn load_data(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    fn load_data_range(&mut self, data: &[u8], start_idx: usize) {
        self.memory[start_idx..start_idx + data.len()].copy_from_slice(data);
    }

    /// Return the state of the display
    #[must_use]
    pub const fn get_display(&self) -> &[bool] {
        &self.display
    }

    /// Press a key 0-15
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        debug_assert!(idx < NUM_KEYS, "{idx} is outside bounds");
        self.keyboard[idx] = pressed;
    }

    const fn read_opcode(&mut self) -> u16 {
        let op_byte_1 = self.memory[self.program_counter as usize] as u16;
        let op_byte_2 = self.memory[(self.program_counter + 1) as usize] as u16;
        let op_code = (op_byte_1 << 8) | op_byte_2;
        self.program_counter += OPCODE_SIZE;
        op_code
    }

    pub const fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // PLAY SOUND
            }
            self.sound_timer -= 1;
        }
    }

    pub fn tick(&mut self) -> Option<()> {
        let opcode = self.read_opcode();

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
        let d = (opcode & 0x000F) as u8;

        let addr = opcode & 0x0FFF;
        let byte = (opcode & 0x00FF) as u8;

        match (c, x, y, d) {
            (0, 0, 0, 0) => {
                return None;
            }
            (0, 0, 0xE, 0) => self.cls(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (1, _, _, _) => self.jmp(addr),
            (2, _, _, _) => self.call(addr),
            (3, _, _, _) => self.skip_val_eq(x, byte),
            (4, _, _, _) => self.skip_val_not_eq(x, byte),
            (5, _, _, _) => self.skip_registers_eq(x, y),
            (6, _, _, _) => self.load_register(x, byte),
            (7, _, _, _) => self.add_to_register(x, byte),
            (8, _, _, 0) => self.load(x, y),
            (8, _, _, 1) => self.or(x, y),
            (8, _, _, 2) => self.and(x, y),
            (8, _, _, 3) => self.xor(x, y),
            (8, _, _, 4) => self.add_xy(x, y),
            (8, _, _, 5) => self.sub_xy(x, y),
            (8, _, _, 6) => self.shift_right(x),
            (8, _, _, 7) => self.subn(x, y),
            (8, _, _, 0xE) => self.shift_left(x),
            (9, _, _, 0) => self.skip_registers_ne(x, y),
            (0xA, _, _, _) => self.load_i_reg(addr),
            (0xB, _, _, _) => self.jump_from(addr),
            (0xC, _, _, _) => self.rand(x, byte),
            (0xD, _, _, _) => self.display(x, y, d),
            (0xE, _, 9, 0xE) => self.skip_if_key(x),
            (0xE, _, 0xA, 1) => self.skip_not_key(x),
            (0xF, _, 0, 7) => self.set_register_to_delay(x),
            (0xF, _, 0, 0xA) => self.wait_timer(x),
            (0xF, _, 1, 5) => self.set_timer(x),
            (0xF, _, 1, 8) => self.set_sound_timer(x),
            (0xF, _, 1, 0xE) => self.add_to_i_register(x),
            (0xF, _, 2, 9) => self.set_i_to_font_addr(x),
            (0xF, _, 3, 3) => self.store_bcd_encoding(x),
            (0xF, _, 5, 5) => self.store_registers_at_i(x),
            (0xF, _, 6, 5) => self.load_registers_from_i_addr(x),
            _ => todo!("opcode {:04x} is not implemented", opcode),
        }
        Some(())
    }

    const fn skip_val_eq(&mut self, x: u8, byte: u8) {
        //  3xkk - SE Vx, byte
        // Skip next instruction if Vx = kk.
        //The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
        if self.v_registers[x as usize] == byte {
            self.program_counter += OPCODE_SIZE;
        }
    }

    const fn skip_val_not_eq(&mut self, x: u8, byte: u8) {
        // 4xkk - SNE Vx, byte
        // Skip next instruction if Vx != kk.
        // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
        if self.v_registers[x as usize] != byte {
            self.program_counter += OPCODE_SIZE;
        }
    }

    const fn skip_registers_eq(&mut self, x: u8, y: u8) {
        // 5xy0 - SE Vx, Vy
        // Skip next instruction if Vx = Vy.
        // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
        if self.v_registers[x as usize] == self.v_registers[y as usize] {
            self.program_counter += OPCODE_SIZE;
        }
    }

    const fn load_register(&mut self, x: u8, byte: u8) {
        // 6xkk - LD Vx, byte
        // Set Vx = kk.
        //The interpreter puts the value kk into register Vx.
        self.v_registers[x as usize] = byte;
    }

    const fn add_to_register(&mut self, x: u8, byte: u8) {
        // 7xkk - ADD Vx, byte
        // Set Vx = Vx + kk.
        // Adds the value kk to the value of register Vx, then stores the result in Vx.
        self.v_registers[x as usize] = self.v_registers[x as usize].wrapping_add(byte);
    }

    const fn load(&mut self, x: u8, y: u8) {
        // 8xy0 - LD Vx, Vy
        // Set Vx = Vy.
        // Stores the value of register Vy in register Vx.
        self.v_registers[x as usize] = self.v_registers[y as usize];
    }

    const fn or(&mut self, x: u8, y: u8) {
        // 8xy1 - OR Vx, Vy
        // Set Vx = Vx OR Vy.
        // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
        self.v_registers[x as usize] |= self.v_registers[y as usize];
    }
    const fn and(&mut self, x: u8, y: u8) {
        // 8xy2 - AND Vx, Vy
        // Set Vx = Vx AND Vy.
        // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
        self.v_registers[x as usize] &= self.v_registers[y as usize];
    }
    const fn xor(&mut self, x: u8, y: u8) {
        //8xy3 - XOR Vx, Vy
        // Set Vx = Vx XOR Vy.
        // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
        self.v_registers[x as usize] ^= self.v_registers[y as usize];
    }
    const fn sub_xy(&mut self, x: u8, y: u8) {
        //8xy5 - SUB Vx, Vy
        //Set Vx = Vx - Vy, set VF = NOT borrow.
        // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
        let arg1 = self.v_registers[x as usize];
        let arg2 = self.v_registers[y as usize];
        let (val, overflow) = arg1.overflowing_sub(arg2);
        self.v_registers[x as usize] = val;
        if overflow {
            self.v_registers[0xF] = 1;
        } else {
            self.v_registers[0xF] = 0;
        }
    }

    const fn shift_right(&mut self, x: u8) {
        // 8xy6 - SHR Vx {, Vy}
        // Set Vx = Vx SHR 1.
        // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
        let lsb = self.v_registers[x as usize] & 1;
        self.v_registers[x as usize] >>= 1;
        self.v_registers[0xF] = lsb;
    }

    const fn subn(&mut self, x: u8, y: u8) {
        //8xy7 - SUBN Vx, Vy
        //Set Vx = Vy - Vx, set VF = NOT borrow.
        //If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
        let arg1 = self.v_registers[x as usize];
        let arg2 = self.v_registers[y as usize];
        let (val, overflow) = arg2.overflowing_sub(arg1);
        self.v_registers[x as usize] = val;
        if overflow {
            self.v_registers[0xF] = 1;
        } else {
            self.v_registers[0xF] = 0;
        }
    }

    const fn shift_left(&mut self, x: u8) {
        // 8xyE - SHL Vx {, Vy}
        // Set Vx = Vx SHL 1.
        //If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
        let msb = (self.v_registers[x as usize] >> 7) & 1;
        self.v_registers[x as usize] <<= 1;
        self.v_registers[0xF] = msb;
    }

    const fn skip_registers_ne(&mut self, x: u8, y: u8) {
        //9xy0 - SNE Vx, Vy
        //Skip next instruction if Vx != Vy.
        //The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
        if self.v_registers[x as usize] != self.v_registers[y as usize] {
            self.program_counter += OPCODE_SIZE;
        }
    }

    const fn cls(&mut self) {
        // 00E0 - CLS
        // Clear the display.
        self.display = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
    }

    const fn jmp(&mut self, addr: u16) {
        // 1nnn - JP addr
        // Jump to location nnn.
        //  The interpreter sets the program counter to nnn.
        self.program_counter = addr;
    }

    fn call(&mut self, addr: u16) {
        // 2nnn - CALL addr
        // Call subroutine at nnn.
        // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
        assert!(self.stack_pointer < self.stack.len(), "Stack overflow");

        self.stack[self.stack_pointer] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = addr;
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
        let (val, overflow) = arg1.overflowing_add(arg2);
        self.v_registers[x as usize] = val;
        if overflow {
            self.v_registers[0xF] = 1;
        } else {
            self.v_registers[0xF] = 0;
        }
    }
    const fn load_i_reg(&mut self, addr: u16) {
        // Annn - LD I, addr
        // Set I = nnn.
        // The value of register I is set to nnn.
        self.i_register = addr;
    }

    const fn jump_from(&mut self, addr: u16) {
        // Bnnn - JP V0, addr
        // Jump to location nnn + V0.
        // The program counter is set to nnn plus the value of V0.
        self.program_counter = (self.v_registers[0] as u16) + addr;
    }

    fn rand(&mut self, x: u8, byte: u8) {
        // Cxkk - RND Vx, byte
        // Set Vx = random byte AND kk.
        // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
        // The results are stored in Vx. See instruction 8xy2 for more information on AND.
        let r = fastrand::u8(..);
        self.v_registers[x as usize] = r & byte;
    }

    fn display(&mut self, x: u8, y: u8, d: u8) {
        // Dxyn - DRW Vx, Vy, nibble
        // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
        // The interpreter reads n bytes from memory, starting at the address stored in I.
        // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
        // Sprites are XORed onto the existing screen.
        // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
        // If the sprite is positioned so part of it is outside the coordinates of the display,
        // it wraps around to the opposite side of the screen.
        // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.

        // Implementation based on: <https://aquova.net/emudev/chip8/5-instr.html>
        let x_coord = self.v_registers[x as usize] as u16;
        let y_coord = self.v_registers[y as usize] as u16;

        let num_rows = u16::from(d);
        let mut flipped = false;
        // Iterate over each row of our sprite
        for y_line in 0..num_rows {
            // Determine which memory address our row's data is stored
            let addr = self.i_register + y_line;
            let pixels = self.memory[addr as usize];
            // Iterate over each column in our row
            for x_line in 0..8 {
                // Use a mask to fetch current pixel's bit. Only flip if a 1
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    // Sprites should wrap around screen, so apply modulo
                    let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                    // Get our pixel's index for our 1D screen array
                    let idx = x + SCREEN_WIDTH * y;
                    // Check if we're about to flip the pixel and set
                    flipped |= self.display[idx];
                    self.display[idx] ^= true;
                }
            }
        }
        // Populate VF register
        if flipped {
            self.v_registers[0xF] = 1;
        } else {
            self.v_registers[0xF] = 0;
        }
    }

    const fn skip_if_key(&mut self, x: u8) {
        // Ex9E - SKP Vx
        // Skip next instruction if key with the value of Vx is pressed.
        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
        let vx = self.v_registers[x as usize];
        let key_press = self.keyboard[vx as usize];
        if key_press {
            self.program_counter += OPCODE_SIZE;
        }
    }

    const fn skip_not_key(&mut self, x: u8) {
        // ExA1 - SKNP Vx
        // Skip next instruction if key with the value of Vx is not pressed.
        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
        let vx = self.v_registers[x as usize];
        let key_press = self.keyboard[vx as usize];
        if !key_press {
            self.program_counter += OPCODE_SIZE;
        }
    }

    const fn set_register_to_delay(&mut self, x: u8) {
        // Fx07 - LD Vx, DT
        // Set Vx = delay timer value.
        // The value of DT is placed into Vx.
        self.v_registers[x as usize] = self.delay_timer;
    }

    fn wait_timer(&mut self, x: u8) {
        // Fx0A - LD Vx, K
        // Wait for a key press, store the value of the key in Vx.
        // All execution stops until a key is pressed, then the value of that key is stored in Vx.
        let mut is_pressed = false;
        for (idx, pressed) in self.keyboard.iter().enumerate() {
            if *pressed {
                self.v_registers[x as usize] = idx as u8;
                is_pressed = true;
                break;
            }
        }
        if !is_pressed {
            self.program_counter -= OPCODE_SIZE;
        }
    }

    const fn set_timer(&mut self, x: u8) {
        // Fx15 - LD DT, Vx
        // Set delay timer = Vx.
        // DT is set equal to the value of Vx.
        self.delay_timer = self.v_registers[x as usize];
    }

    const fn set_sound_timer(&mut self, x: u8) {
        // Fx18 - LD ST, Vx
        // Set sound timer = Vx.
        // ST is set equal to the value of Vx.
        self.sound_timer = self.v_registers[x as usize];
    }

    const fn add_to_i_register(&mut self, x: u8) {
        // Fx1E - ADD I, Vx
        // Set I = I + Vx.
        // The values of I and Vx are added, and the results are stored in I.
        self.i_register += self.v_registers[x as usize] as u16;
    }

    const fn set_i_to_font_addr(&mut self, x: u8) {
        // Fx29 - LD F, Vx
        // Set I = location of sprite for digit Vx.
        // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
        // See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
        let char = self.v_registers[x as usize] as u16;
        // Every hex char is 5 bytes
        self.i_register = char * 5;
    }

    fn store_bcd_encoding(&mut self, x: u8) {
        // Fx33 - LD B, Vx
        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
        // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
        // the tens digit at location I+1, and the ones digit at location I+2.
        // https://en.wikipedia.org/wiki/Binary-coded_decimal
        let vx = self.v_registers[x as usize] as f64;

        let hundredths = (vx / 100.0).floor() as u8;
        let tenths = ((vx / 10.0) % 10.0).floor() as u8;
        let ones = (vx % 1.0).floor() as u8;

        self.load_data_range(&[hundredths, tenths, ones], self.i_register as usize);
    }

    fn store_registers_at_i(&mut self, x: u8) {
        // Fx55 - LD [I], Vx
        // Store registers V0 through Vx in memory starting at location I.
        // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
        let i_addr = self.i_register as usize;
        for offset in 0..=x as usize {
            self.memory[i_addr + offset] = self.v_registers[x as usize];
        }
    }

    fn load_registers_from_i_addr(&mut self, x: u8) {
        // Fx65 - LD Vx, [I]
        // Read registers V0 through Vx from memory starting at location I.
        // The interpreter reads values from memory starting at location I into registers V0 through Vx.
        for reg_idx in 0..=x as usize {
            self.v_registers[reg_idx] = self.memory[self.i_register as usize + reg_idx];
        }
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

        let data: [u8; 6] = [
            0x21, 0x00, // Call (0x100)
            0x21, 0x00, // Call (0x100)
            0x00, 0x00, // End
        ];
        cpu.load_data(&data);

        let func_data: [u8; 6] = [
            0x80, 0x14, // Add(0, 1)
            0x80, 0x14, // Add(0, 1)
            0x00, 0xEE, // End
        ];
        cpu.load_data_range(&func_data, 0x100);
        loop {
            if cpu.tick().is_none() {
                break;
            }
        }
        assert_eq!(cpu.v_registers[0], 45);
    }

    #[test]
    fn load_rom_pong() {
        let mut cpu = Chip8Emulator::new();
        let bytes = include_bytes!("./roms/PONG");
        cpu.load_data(bytes);
        let mut counter = 0;
        while counter < 10000 {
            if cpu.tick().is_none() {
                break;
            }
            cpu.tick_timers();
            counter += 1;
        }
    }

    #[test]
    fn load_rom_guess() {
        let mut cpu = Chip8Emulator::new();
        let bytes = include_bytes!("./roms/GUESS");
        cpu.load_data(bytes);
        let mut counter = 0;
        while counter < 10000 {
            if cpu.tick().is_none() {
                break;
            }
            cpu.tick_timers();
            counter += 1;
        }
    }

    #[test]
    fn load_rom_maze() {
        let mut cpu = Chip8Emulator::new();
        let bytes = include_bytes!("./roms/MAZE");
        cpu.load_data(bytes);
        let mut counter = 0;
        while counter < 10000 {
            if cpu.tick().is_none() {
                break;
            }
            cpu.tick_timers();
            counter += 1;
        }
    }
}
