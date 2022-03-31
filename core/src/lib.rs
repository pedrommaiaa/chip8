use rand::random;

// Chip-8 uses a 64X32 monochrome (1 bit per pixel) display.
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200; // first 512 store font data
const RAM_SIZE: usize = 4096;  // 4KB
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

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
    pc: u16,                                      // Program Counter
    ram: [u8; RAM_SIZE],                          // RAM, array of 4096 bytes, 8-bit (1 byte) each.
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // handle screen display.
    v_reg: [u8; NUM_REGS],                        // 8-bit registers (V0..VF), used in game, fast.
    i_reg: u16,                                   // register used for indexing into RAM for reads and writes.
    sp: u16,                                      // Stack Pointer, top of the stack
    stack: [u16; STACK_SIZE],                     // stack, FIFO principle, useful on subrutines
    keys: [bool; NUM_KEYS],
    dt: u8,                                       // Delay Timer, tipical timer, when 0 is performs some action
    st: u8,                                       // Sound Timer,  when 0 emits a noise.
}

impl Cpu {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            pc: START_ADDR, // 512, beginning of the program
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_cpu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_cpu
    }
    
    // Used to render the display on the frontend
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }
    
    // Handles key presses
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // Load the game from file
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR; 
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp =  0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0; 
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
    
    fn push(&mut self, val:u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 { 
        // add protection agaist underflow panic
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
    
    fn fetch(&mut self) -> u16 {
        // Grab the instruction we are about to execute
        // fetches the 16-bit opcode stored at outr current pc. we store values
        // in RAM as 8-bit, so we fetch two and combine them as Big Endian.
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        // todo: Implement sound
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // 00E0, NOP
            (0, 0, 0xE, 0) => { // CLS, 0x00E0 is the opcode to clean the screen
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            (0, 0, 0xE,0xE) => { // 00EE, RET, Return from subroutine (functions). 
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            (1, _, _, _) => { // 1NNN, JMP NNN, Jump instrction, move PC to given address 
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            (2, _, _, _) => { // 2NNN, CALL NNN, Call subroutine, add to the stack.. 
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            (3, _, _, _) => { // 3XNN, Skip next if VX == NN, if-else functionality
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => { // 4XNN, Skip next if VX != NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => { // 5XY0, Skip VX == VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => { // 6XNN, VX = NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            (7, _, _, _) => { // 7XNN, VX += NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            (8, _, _, 0) => { // 8XY0, VX = VY, like VX=NN but source value is from VY register.
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            (8, _, _, 1) => { // 8XY1, OR Bitwise operation, VX |= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            (8, _, _, 2) => { // 8XY2, AND Bitwise operation, VX &= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            (8, _, _, 3) => { // 8XY3, XOR Bitwise operation, VX ^= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            (8, _, _, 4) => { // 8XY4, VX += VY, can overflow, 
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 5) => { // 8XY5, VX -= VY, can underflow.
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
                
            },
            (8, _, _, 6) => { // 8XY6, VX >>= 1.
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            (8, _, _, 7) => { // 8XY7, VX = VY - VX
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 0xE) => { // 8XYE, VX <<= 1
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            (9, _, _, 0) => { // 9XY0, SKIP if VX != VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => { // ANNN, I = ANNN
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            (0xB, _, _, _) => { // BNNN, JMP V0 + NNN
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            (0xC, _, _, _) => { // CXNN, VX = rand() & NN 
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            (0xD, _, _, _) => { // DXYN, Draw Sprite
                // Get the (x, y) coords for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;
                // Keep track if any pixels were flipped
                let mut flipped = false;
                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b10000000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            (0xE, _, 9, 0xE) => { // EX9E, Skip if Key Pressed
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => { // EXA1, Skip if Key Not Pressed
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => { // FX07, VX = DT
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            (0xF, _, 0, 0xA) => { // FX0A, Wait for Key Press
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 1, 5) => { // FX15, DT = VX
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            (0xF, _, 1, 8) => { // FX18, ST = VX
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            (0xF, _, 1, 0xE) => { //  FX1E, I += VX
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            (0xF, _, 2, 9) => { // FX29, Set I to Font Address
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            (0xF, _, 3, 3) => { // FX33, I = BCD of VX
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                // Fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Fetch the tens digit by dividing by 10, tossing the one digit and the decimal
                let tens = ((vx/10.0) % 10.0) as u8;
                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            (0xF, _, 5, 5) => { // FX55, Store V0 - VX into I
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i+idx] = self.v_reg[idx];
                }
            },
            (0xF, _, 6, 5) => { // FX65, Load I into V0 - VX
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i+idx];
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemeted opcode: {}", op),
        }
    }
}
