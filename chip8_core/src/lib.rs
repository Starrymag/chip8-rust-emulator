use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const STACK_SIZE: usize = 16;
const FONTSET_SIZE: usize = 80;
const START_ADDR: u16 = 0x200;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Cpu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
    need_beep: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            need_beep: false,
        };
        new_cpu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_cpu
    }
    
    pub fn reest(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();
        // decode and execute
        self.execute(op);
        //writeback
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
            self.need_beep = true;
        } else {
            self.need_beep = false;
        }
    }

    pub fn get_beep_status(&self) -> bool {
        self.need_beep
    }

    pub fn get_display(&self) -> &[bool]{
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start_addr = START_ADDR as usize;
        let end_addr = (START_ADDR as usize) + data.len();
        self.ram[start_addr..end_addr].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        // decode
        let f1 = (op & 0xF000) >> 12;
        let f2 = (op & 0x0F00) >> 8;
        let f3 = (op & 0x00F0) >> 4;
        let f4 = op & 0x000F;

        // prefetch most common used fields
        let x = self.v_reg[f2 as usize];
        let y = self.v_reg[f3 as usize];
        let x_ptr = &mut self.v_reg[f2 as usize];
        let nnn = op & 0xFFF;
        let kk = (op & 0xFF) as u8;

        match (f1, f2, f3, f4) {
            // NOP
            (0, 0, 0, 0) => return,

            // CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },

            // RET
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },

            // JMP nnn
            (1, _, _, _) => {
                self.pc = nnn;
            },

            // CALL nnn
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = nnn;
            },

            // SE
            (3, _, _, _) => {
                if x == kk {
                    self.pc += 2;
                }
            },

            // SNE
            (4, _, _, _) => {
                if x != kk {
                    self.pc += 2;
                }
            },


            // SE
            (5, _, _, 0) => {
                if x == y {
                    self.pc += 2;
                }
            },

            // LD
            (6, _, _, _) => {
                *x_ptr = kk;
            },

            // ADD
            (7, _, _, _) => {
                *x_ptr = x.wrapping_add(kk);
            },

            // LD Vx Vy
            (8, _, _, 0) => {
                *x_ptr = y;
            },

            // OR Vx Vy
            (8, _, _, 1) => {
                *x_ptr = x | y;
            },

            // AND Vx Vy
            (8, _, _, 2) => {
                *x_ptr = x & y;
            },

            // XOR Vx Vy
            (8, _, _, 3) => {
                *x_ptr = x ^ y;
            },

            // ADD Vx Vy
            (8, _, _, 4) => {
                let (new_x, carry) = x.overflowing_add(y);
                let new_f = if carry { 1 } else { 0 };

                *x_ptr = new_x;
                self.v_reg[0xF] = new_f;
            },


            // SUB Vx Vy
            (8, _, _, 5) => {
                let (new_x, borrow) = x.overflowing_sub(y);
                let new_f = if borrow { 0 } else { 1 };

                *x_ptr = new_x;
                self.v_reg[0xF] = new_f;
            },

            // SHR
            (8, _, _, 6) => {
                let lsb = x & 1;
                *x_ptr = x >> 1;
                self.v_reg[0xF] = lsb;
            },

            // SUBN Vx Vy
            (8, _, _, 7) => {
                let (new_x, borrow) = y.overflowing_sub(x);
                let new_f = if borrow { 0 } else { 1 };

                *x_ptr = new_x;
                self.v_reg[0xF] = new_f;
            },

            // SHR
            (8, _, _, 0xE) => {
                let msb = (x >> 7) & 1;
                *x_ptr = x << 1;
                self.v_reg[0xF] = msb;
            },

            // SNE Vx Vy
            (9, _, _, 0) => {
                if x != y {
                    self.pc += 2;
                }
            },

            // SET I
            (0xA, _, _, _) => {
                self.i_reg = nnn;
            },


            // JUMP
            (0xB, _, _, _) => {
                self.pc = (self.v_reg[0x0] as u16) + nnn;
            },

            // RND
            (0xC, _, _, _) => {
                let rnd: u8 = random();
                *x_ptr = rnd & kk;
            },

            // DRAW 
            (0xD, _, _, _) => {
                let num_bytes_to_read = f4;
                let mut flipped = false;
                for y_line in 0..num_bytes_to_read {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x_coord = (x as u16 + x_line) as usize % SCREEN_WIDTH;
                            let y_coord = (y as u16 + y_line) as usize % SCREEN_HEIGHT;
                            let idx = x_coord + SCREEN_WIDTH * y_coord;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }

            },

            // SKIP if key pressed
            (0xE, _, 9, 0xE) => {
                let key = self.keys[x as usize];
                if key {
                    self.pc += 2;
                }
            },

            // SKIP if key not pressed
            (0xE, _, 0xA, 1) => {
                let key = self.keys[x as usize];
                if !key {
                    self.pc += 2;
                }
            },

            // LD
            (0xF, _, 0, 7) => {
                *x_ptr = self.dt;
            },

            // WAIT for key pressed
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        *x_ptr = i as u8;
                        pressed = true;
                        break;
                    }
                }
                
                if !pressed {
                    self.pc -= 2;
                }
            },

            // LD dt
            (0xF, _, 1, 5) => {
                self.dt = x;
            },

            // LD st
            (0xF, _, 1, 8) => {
                self.st = x;
            },

            // ADD
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(x as u16);
            },

            // I = FONT
            (0xF, _, 2, 9) => {
                self.i_reg = (x as u16) * 5;
            },

            // BCD
            (0xF, _, 3, 3) => {
                let hundreds = (x as f32 / 100.0).floor() as u8;
                let tens = ((x as f32 / 10.0) % 10.0).floor() as u8;
                let ones = (x % 10) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },

            // STORE V0 to VX
            (0xF, _, 5, 5) => {
                let i = self.i_reg as usize;
                for idx in 0..=f2 as usize {
                    self.ram[i + idx as usize] = self.v_reg[idx as usize];
                }
                self.i_reg += (x + 1) as u16;
            },

            // STORE V0 to VX
            (0xF, _, 6, 5) => {
                let i = self.i_reg as usize;
                for idx in 0..=f2 as usize {
                    self.v_reg[idx as usize] = self.ram[i + idx as usize] 
                }
                self.i_reg += (x + 1) as u16;
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode {}", op),
        }
    }
}
