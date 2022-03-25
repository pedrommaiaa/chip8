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
    
    fn push(&mut self, val:u16) {}

    fn pop(&mut self) -> u16 {}
    
    fn fetch(&mut self) -> u16 {}

    pub fn tick_timers(&mut self) {}

    pub fn tick(&mut self) {}

    fn execute(&mut self, op: u16) {}
}
