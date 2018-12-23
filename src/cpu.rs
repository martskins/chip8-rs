use crate::font::FONT_SET;
use std::io::Read;

const OPCODE_SIZE: u16 = 2;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM: usize = 4096;

enum ProgramCounter {
    Next,
    Skip,
    Jump(u16),
}

impl ProgramCounter {
    fn skip_if(cond: bool) -> ProgramCounter {
        if cond {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }
}

pub struct CPU {
    pub opcode: u16,
    pub i: u16,
    pub pc: u16,
    pub memory: [u8; RAM],
    pub v: [u8; 16],
    pub stack: [u16; 16],
    pub sp: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub draw_screen: bool,
    pub keypad: [bool; 16],
    pub keypad_waiting: bool,
    pub keypad_register: u16,
}

impl CPU {
    pub fn new() -> CPU {
        let mut mem = [0; RAM];
        for i in 0..FONT_SET.len() {
            mem[i] = FONT_SET[i];
        }

        CPU {
            opcode: 0,
            i: 0x200,
            pc: 0x200,
            memory: mem,
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            draw_screen: false,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        }
    }

    pub fn fetch_and_process_opcode(&mut self) {
        let at = self.pc as usize;
        let opcode = (self.memory[at] as u16) << 8 | (self.memory[at + 1] as u16);
        self.opcode = opcode;
        self.process_opcode();
    }

    pub fn tick(&mut self, keypad: [bool; 16]) {
        self.draw_screen = false;
        self.keypad = keypad;
        if self.keypad_waiting {
            let keypad = self.keypad;
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypad_waiting = false;
                    self.v[self.keypad_register as usize] = i as u8;
                    break;
                }
            }
        } else {
            self.fetch_and_process_opcode();
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }
        for i in 0..self.keypad.len() {
            if self.keypad[i] {
                println!("{:?}", i);
            }
        }
        // self.keypad = [false; 16];
    }

    fn process_opcode(&mut self) {
        let action = match self.opcode & 0xF000 {
            0x0000 => self.x0nnn(),
            0x1000 => self.x1nnn(),
            0x2000 => self.x2nnn(),
            0x3000 => self.x3xkk(),
            0x4000 => self.x4xkk(),
            0x5000 => self.x5xy0(),
            0x6000 => self.x6xkk(),
            0x7000 => self.x7xkk(),
            0x8000 => self.x8xyn(),
            0x9000 => self.x9xy0(),
            0xA000 => self.xannn(),
            0xB000 => self.xbnnn(),
            0xC000 => self.xcxkk(),
            0xD000 => self.xdxyn(),
            0xE000 => self.xennn(),
            0xF000 => self.xfnnn(),
            _ => ProgramCounter::Next,
        };

        match action {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Skip => self.pc += 2 * OPCODE_SIZE,
            ProgramCounter::Jump(addr) => self.pc = addr as u16,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut rom = std::fs::File::open(path).unwrap();
        let mut bts: Vec<u8> = vec![];
        rom.read_to_end(&mut bts).unwrap();
        self.load_program(bts)
    }

    fn load_program(&mut self, program: Vec<u8>) {
        for (idx, &byte) in program.iter().enumerate() {
            self.memory[0x200 + idx] = byte;
        }
    }

    fn x0nnn(&mut self) -> ProgramCounter {
        match self.opcode & 0x00FF {
            0x00E0 => self.x00e0(),
            0x00EE => self.x00ee(),
            _ => panic!("not implemented"),
        }
    }

    // CLS: Clear the screen
    fn x00e0(&mut self) -> ProgramCounter {
        self.display = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.draw_screen = true;
        ProgramCounter::Next
    }

    // RTS: Return from subroutine
    fn x00ee(&mut self) -> ProgramCounter {
        self.sp -= 1;
        ProgramCounter::Jump(self.stack[self.sp as usize])
    }

    // JMP nnn: Jumps to address nnn
    fn x1nnn(&mut self) -> ProgramCounter {
        ProgramCounter::Jump(self.op_0nnn())
    }

    // JSR nnn: Jump to subroutine at address nnn
    fn x2nnn(&mut self) -> ProgramCounter {
        self.stack[self.sp as usize] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        ProgramCounter::Jump(self.op_0nnn())
    }

    // SKEQ vx, kk: Skip if register x == kk
    fn x3xkk(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let kk = self.op_00nn() as u8;
        ProgramCounter::skip_if(self.v[x] == kk)
    }

    // SKNE vx, kk: Skip if register x != kk
    fn x4xkk(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let kk = self.op_00nn() as u8;
        ProgramCounter::skip_if(self.v[x] != kk)
    }

    // SKEQ vx, vy: Skip if register x == register y
    fn x5xy0(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let y = self.op_00n0() as usize;
        ProgramCounter::skip_if(self.v[x] == self.v[y])
    }

    // MOV vx, kk: Move kk to register x
    fn x6xkk(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let kk = self.op_00nn() as u8;
        self.v[x] = kk;
        ProgramCounter::Next
    }

    // ADD vx, kk: Add kk to register x
    fn x7xkk(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let kk = self.op_00nn() as u16;
        let vx = self.v[x] as u16;
        let res = vx + kk;
        self.v[x] = res as u8;
        ProgramCounter::Next
    }

    fn x8xyn(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let y = self.op_00n0() as usize;
        let val = match self.opcode & 0x000F {
            0x0000 => self.x8xy0(x, y),
            0x0001 => self.x8xy1(x, y),
            0x0002 => self.x8xy2(x, y),
            0x0003 => self.x8xy3(x, y),
            0x0004 => self.x8xy4(x, y),
            0x0005 => self.x8xy5(x, y),
            0x0006 => self.x8xy6(x, y),
            0x0007 => self.x8xy7(x, y),
            0x000E => self.x8xye(x, y),
            _ => panic!("not implemented"),
        };
        self.v[x as usize] = val;
        ProgramCounter::Next
    }

    // MOV vx, vy: Move register y to register x
    fn x8xy0(&mut self, _x: usize, y: usize) -> u8 {
        self.v[y]
    }

    // OR vx, vy: Bitwise OR register y into register x
    fn x8xy1(&mut self, x: usize, y: usize) -> u8 {
        self.v[x] | self.v[y]
    }

    // AND vx, vy: Bitwise AND register y into register x
    fn x8xy2(&mut self, x: usize, y: usize) -> u8 {
        self.v[x] & self.v[y]
    }

    // XOR vx, vy: Bitwise XOR register y into register x
    fn x8xy3(&mut self, x: usize, y: usize) -> u8 {
        self.v[x] ^ self.v[y]
    }

    // ADD vx, vy: Add register y to register x, carry into register f
    fn x8xy4(&mut self, x: usize, y: usize) -> u8 {
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        self.v[0x0F] = if vx + vy > 0xFF { 1 } else { 0 };
        (0x00FF & (vx + vy)) as u8
    }

    // SUB vx, vy: Subtract register y from register x, borrow in register f
    fn x8xy5(&mut self, x: usize, y: usize) -> u8 {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[0x0F] = if vx > vy { 1 } else { 0 };
        vx.wrapping_sub(vy)
    }

    // SHR vx: Shift register y right, bit 0 goes into register f
    fn x8xy6(&mut self, x: usize, _y: usize) -> u8 {
        self.v[0x0F] = self.v[x] & 0x0001;
        self.v[x] >> 1
    }

    // RSB vx, vy: Subtract register x from register y, result in register x
    fn x8xy7(&mut self, x: usize, y: usize) -> u8 {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[0x0F] = if vy > vx { 1 } else { 0 };
        vy.wrapping_sub(vx)
    }

    // SHL vx: Shift register x left, bit 7 goes into register f
    fn x8xye(&mut self, x: usize, _y: usize) -> u8 {
        self.v[0x0F] = (self.v[x] & 0b1000_0000) >> 7;
        self.v[x] << 1
    }

    // SKNE vx, vy: Skip if register x != register y
    fn x9xy0(&mut self) -> ProgramCounter {
        let x = self.v[self.op_0n00() as usize] as usize;
        let y = self.v[self.op_00n0() as usize] as usize;
        ProgramCounter::skip_if(self.v[x] != self.v[y])
    }

    // MVI nnn: Load index register with constant nnn
    fn xannn(&mut self) -> ProgramCounter {
        self.i = self.op_0nnn();
        ProgramCounter::Next
    }

    // JMI nnn: Jump to address nnn + register 0
    fn xbnnn(&mut self) -> ProgramCounter {
        let addr = self.op_0nnn() + (self.v[0] as u16);
        ProgramCounter::Jump(addr)
    }

    // RAND vx, kk: Random number less than kk into register x
    fn xcxkk(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let kk = self.op_00nn();
        let r = rand::random::<u8>();
        self.v[x] = r & (kk as u8);
        ProgramCounter::Next
    }

    // SPRITE vx, vy, n: Draw sprite at screen location vx, vy, height s
    fn xdxyn(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        let y = self.op_00n0() as usize;
        let n = self.op_000n() as usize;
        self.v[0x0f] = 0;
        for byte in 0..n {
            let y = (self.v[y] as usize + byte) % SCREEN_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % SCREEN_WIDTH;
                let color = (self.memory[(self.i as usize) + byte] >> (7 - bit)) & 1;
                self.v[0x0f] |= color & self.display[y][x];
                self.display[y][x] ^= color;
            }
        }

        // =========================================== //

        // let n = self.op_000n();
        // let x = self.op_0n00() as usize;
        // let y = self.op_00n0() as usize;
        // for i in 0..n {
        //     let i = (self.i as usize) + (i as usize);
        //     self.display[y][x] ^= self.memory[i];
        // }
        self.draw_screen = true;
        ProgramCounter::Next
    }

    fn xennn(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        match self.opcode & 0x00FF {
            0x009E => self.xex9e(x),
            0x00A1 => self.xexa1(x),
            _ => panic!("unknown instruction"),
        }
    }

    // SKIPR x: Skip if key x is pressed
    fn xex9e(&mut self, x: usize) -> ProgramCounter {
        let vx = self.v[x] as usize;
        ProgramCounter::skip_if(self.keypad[vx])
    }

    // SKUP x: Skip if key x is not pressed
    fn xexa1(&mut self, x: usize) -> ProgramCounter {
        let vx = self.v[x] as usize;
        ProgramCounter::skip_if(!self.keypad[vx])
    }

    fn xfnnn(&mut self) -> ProgramCounter {
        let x = self.op_0n00() as usize;
        match self.opcode & 0x00FF {
            0x0007 => self.xfx07(x),
            0x000A => self.xfx0a(x),
            0x0015 => self.xfx15(x),
            0x0018 => self.xfx18(x),
            0x001E => self.xfx1e(x),
            0x0029 => self.xfx29(x),
            0x0030 => self.xfx30(x),
            0x0033 => self.xfx33(x),
            0x0055 => self.xfx55(x),
            0x0065 => self.xfx65(x),
            _ => panic!("not implemented!!"),
        }
        ProgramCounter::Next
    }

    fn xfx07(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
    }

    fn xfx0a(&mut self, x: usize) {
        self.keypad_waiting = true;
        self.keypad_register = x as u16;
    }

    fn xfx15(&mut self, x: usize) {
        self.delay_timer = self.v[x];
    }

    fn xfx18(&mut self, x: usize) {
        self.sound_timer = self.v[x];
    }

    fn xfx1e(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        self.v[0x0F] = if self.i > 0x0F00 { 1 } else { 0 };
    }

    fn xfx29(&mut self, x: usize) {
        self.i = ((self.v[x] as usize) * 5) as u16;
    }

    // xfont vx: Point the index register to the sprite for hex in register x
    // Superchip only
    fn xfx30(&mut self, x: usize) {
        self.i = ((self.v[x] as usize) * 10) as u16;
    }

    // BCD vx: Store the bcd representation of register x at location i, i+1, i+2
    fn xfx33(&mut self, x: usize) {
        let vx = self.v[x];
        let i = self.i as usize;
        self.memory[i] = vx / 100;
        self.memory[i + 1] = (vx % 100) / 10;
        self.memory[i + 2] = vx % 10;
    }

    fn xfx55(&mut self, x: usize) {
        for i in 0..=x {
            let at = (self.i + i as u16) as usize;
            self.memory[at] = self.v[i as usize];
        }
    }

    fn xfx65(&mut self, x: usize) {
        for i in 0..=x {
            let at = (self.i + i as u16) as usize;
            self.v[i] = self.memory[at];
        }
    }

    fn op_0n00(&self) -> u16 {
        (self.opcode & 0x0f00) >> 8
    }
    fn op_00n0(&self) -> u16 {
        (self.opcode & 0x00f0) >> 4
    }
    fn op_000n(&self) -> u16 {
        (self.opcode & 0x000f)
    }
    fn op_00nn(&self) -> u16 {
        self.opcode & 0x00ff
    }
    fn op_0nnn(&self) -> u16 {
        self.opcode & 0x0fff
    }
}

#[cfg(test)]
#[path = "./cpu_test.rs"]
mod cpu_test;
