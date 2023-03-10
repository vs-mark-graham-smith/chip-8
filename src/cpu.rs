use std::fs;
use std::time::Duration;
use crate::keypad::Keypad;
use crate::display::Display;
use std::thread;

pub struct Cpu {
    program: usize,
    opcode: u16,
    stack: [u16; 16],
    stack_pointer: usize,
    delay_timer: u8,
    sound_timer: u8,

    v: [u8; 16], // cpu registers (V0 through Ee)
    i: usize, // index register

    memory: [u8; 4096], // system memory

    pub keypad: Keypad, // intercept keyboard calls
    pub display: Display, // visualize on screen
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut memory = [0; 4096];

        for i in 0..80 {
            memory[i] = FONTS[i];
        };

        Cpu {
            program: 0x200, // program counter
            opcode: 0,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,

            v: [0; 16],
            i: 0x200,

            memory,
            keypad: Keypad::new(),
            display: Display::new(),
        }
    }

    pub fn load_game(&mut self, game: &str) {
        let buffer = fs::read(game).expect("Unable to locate ROM");

        // load ROM into memory (AFTER system reserved memory)
        for i in 0..buffer.len() {
            self.memory[i + self.program] = buffer[i];
        };
    }

    pub fn cpu_cycle(&mut self) {
        self.fetch_opcode();
        self.execute_opcode();

        if self.delay_timer > 0 { self.delay_timer -= 1; }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 { println!("BEEP!\n"); }
            self.sound_timer -= 1;
        }

        thread::sleep(Duration::from_micros(500));
    }

    pub fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.program as usize] as u16) << 8 | (self.memory[(self.program + 1) as usize] as u16);
    }

    pub fn execute_opcode(&mut self) {
        // println!("opp {:#04x}", self.opcode);
        match self.opcode & 0xf000 {
            0x0000 => {
                match self.opcode & 0x000f {
                    0x0000 => self.display.clear(),
                    0x000E => {
                        self.stack_pointer -= 1;
                        self.program = self.stack[self.stack_pointer] as usize;
                    },
                    _ => {
                        // println!("Not implemented.");

                        // println!("Opcode: {:04x}", self.opcode);

                        // println!("CPU v: {:#?}", self.v);
                        // println!("CPU I: {:#04X}", self.i);

                        // panic!("Panicked");
                        self.program += 2;
                    }
                } 

                self.program += 2;
            } 
            0x1000 => { // JP addr
                self.program = self.op_nnn() as usize;
                // println!("JP {:#04x}", self.op_nnn());
            }
            0x2000 => { // Call addr
                // println!("call {}", self.op_nnn() as usize);
                self.stack[self.stack_pointer] = self.program as u16;
                self.stack_pointer += 1;
                self.program = self.op_nnn() as usize;
            }
            0x3000 => { // SE Vx, byte
                self.program += if self.v[self.op_x()] == self.op_nn() { 4 } else { 2 };
                // println!("SE V{}, {}", self.op_x(), self.op_nn());
            }
            0x4000 => { // 4xkk - SNE Vx, byte
                self.program += if self.v[self.op_x()] != self.op_nn() { 4 } else { 2 }
            }
            // 0x5000 => self.op_5xy0(), // SE Vx, Vy
            0x6000 => { // LD Vx, byte
                // println!("LD V{}, {:#04X}", self.op_x(), self.op_nn());
                self.v[self.op_x()] = self.op_nn();
                self.program += 2;
            }
            0x7000 => { // ADD Vx, byte
               let vx = self.v[self.op_x()] as u16;
               let val = self.op_nn() as u16;
               let result = vx + val;
               self.v[self.op_x()] = result as u8;

                self.program += 2;
            }
            0x8000 => {
                match self.opcode & 0x000f {
                    0x0000 => { // LD Vx, Vy
                        // println!("LD V{}, V{}", self.op_x(), self.op_y());
                        self.v[self.op_x()] = self.v[self.op_y()];
                    }
                    //    0x0001 => self.op_8xy1(), // OR Vx, Vy
                    0x0002 => { // AND Vx, Vy
                        // println!("AND V{}, V{}", self.op_x(), self.op_y());
                        self.v[self.op_x()] &= self.v[self.op_y()];
                    } 
                    //    0x0003 => self.op_8xy3(), // XOR Vx, Vy
                    0x0004 => { // ADD Vx, Vy
                        let (res, overflow) = self.v[self.op_x()].overflowing_add(self.v[self.op_y()]);
                        match overflow {
                            true => self.v[0xF] = 1,
                            false => self.v[0xF] = 0,
                        }
                        self.v[self.op_x()] = res; { 0 };
                    }
                    0x0005 => { // SUB Vx, Vy
                        let (res, overflow) = self.v[self.op_x()].overflowing_sub(self.v[self.op_y()]);
                        match overflow {
                            true => self.v[0xF] = 0,
                            false => self.v[0xF] = 1,
                        }
                        self.v[self.op_x()] = res;
                    }
                    //    0x0006 => self.op_8xy6(), // SHR Vx {, Vy}
                    //    0x0007 => self.op_8xy7(), // SUBN Vx, Vy
                    //    0x000E => self.op_8xyE(), // SHL Vx {, Vy}
                    _ => {
                        // println!("Not implemented.");

                        // println!("Opcode: {:04x}", self.opcode);

                        // println!("CPU v: {:#?}", self.v);
                        // println!("CPU I: {:#04X}", self.i);

                        // panic!("Panicked");
                        //
                        self.program += 2;
                    }
                }

                self.program += 2;
            }
            // 0x9000 => self.op_9xy0(), // SNE Vx, Vy
            0xA000 => { // LD I, addr
                self.i = self.op_nnn() as usize;
                // println!("LD I, {:#04X}", self.i);
                self.program += 2;
            }
            // 0xB000 => self.op_Bnnn(), // JP V0, addr
            0xC000 => { // RND Vx, byte
                let rand = rand::random::<u8>();
                // println!("rand: {}", rand);
                self.v[self.op_x()] = self.op_nn() & rand::random::<u8>();
                self.program += 2;
            }
            // 0xD000 => self.op_Dxyn(), // DRW Vx, Vy, nibble
            0xD000 => { // DRW Vx, Vy, nibble
                let from = self.i;
                let nibble = self.op_n() as usize;
                let to = from + nibble;
                let x = self.v[self.op_x()];
                let y = self.v[self.op_y()];
                //println!("DRW V{}, V{}, nibble (n: {})", self.op_x(), self.op_y(), nibble);

                self.v[0xF] = self.display.draw(x as usize, y as usize, &self.memory[from..to]);
                self.program += 2;
            }
            0xE000 => {
                match self.opcode & 0x00ff {
                    0x009E => { // SKP Vx
                        //println!("SKP V{}", self.op_x());
                        if self.keypad.pressed(self.v[self.op_x()] as usize) {
                            self.program += 2;
                        }
                    }
                    0x00A1 => { // SKNP Vx
                        //println!("SKNP V{}", self.op_x());
                        if ! self.keypad.pressed(self.v[self.op_x()] as usize) {
                            self.program += 2;
                        }
                    }
                    _ => {
                        // println!("Not implemented.");

                        // println!("Opcode: {:04x}", self.opcode);

                        // println!("CPU v: {:#?}", self.v);
                        // println!("CPU I: {:#04X}", self.i);

                        // panic!("Panicked");
                        self.program += 2;
                    }
                }
                self.program += 2;
            } 
            0xF000 => {
                match self.opcode & 0x00ff {
                    0x0007 => { // LD Vx, DT
                        self.v[self.op_x()] = self.delay_timer;
                        // println!("LD V{}, DT", self.op_x());
                    }
                    // 0x000A => self.op_Fx0A(), // LD Vx, K
                    0x0015 => { // LD DT, Vx
                        self.delay_timer = self.v[self.op_x()];
                        // println!("LD DT, V{}", self.op_x());
                    }
                    0x0018 => { // LD ST, Vx
                        self.sound_timer = self.v[self.op_x()];
                        self.program += 2;
                    }
                    // 0x001E => self.op_Fx1E(), // ADD I, Vx
                    0x0029 =>{ // LD F, Vx
                        // println!("LD F, V{}", self.op_x() as usize);
                        self.i = (self.v[self.op_x()] as usize) * 5;
                    }
                    0x0033 => { // LD B, Vx
                        // println!("LD B, Vx");
                        self.memory[self.i] = self.v[self.op_x()] / 100;
                        self.memory[self.i + 1] = (self.v[self.op_x()] / 10) % 10;
                        self.memory[self.i + 2] = (self.v[self.op_x()] % 100) % 10;
                    }
                    0x0055 => { // LD [I], Vx
                        for i in 0..(self.op_x() + 1) {
                            self.memory[self.i + i] = self.v[i]
                        }
                        self.i += self.op_x() + 1;
                    }
                    0x0065 => { // LD Vx, [I]
                        for i in 0..(self.op_x() + 1) {
                            self.v[i] = self.memory[self.i + i]
                        }
                        self.i += self.op_x() + 1;
                    }
                    _ => {
                        // println!("Not implemented.");

                        // println!("Opcode: {:04x}", self.opcode);

                        // println!("CPU v: {:#?}", self.v);
                        // println!("CPU I: {:#04X}", self.i);

                        // panic!("Panicked");
                        self.program += 2;
                    }
                }

                self.program += 2;
            }
            _ => {
                // println!("Not implemented.");

                // println!("Opcode: {:04x}", self.opcode);

                // println!("CPU v: {:#?}", self.v);
                // println!("CPU I: {:#04X}", self.i);

                // panic!("Panicked");
                        self.program += 2;
            }
        }
    }

    /**
     * Fetch relevant bits from opcode.
     *  */
    fn op_x(&self) -> usize {
        ((self.opcode & 0x0f00) >> 8) as usize
    }

    fn op_y(&self) -> usize {
        ((self.opcode & 0x00f0) >> 4) as usize
    }

    fn op_n(&self) -> u8 {
        (self.opcode & 0x000f) as u8
    }

    fn op_nn(&self) -> u8 {
        (self.opcode & 0x00ff) as u8
    }

    fn op_nnn(&self) -> u16 {
        (self.opcode & 0x0fff) as u16
    }
}

static FONTS: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
