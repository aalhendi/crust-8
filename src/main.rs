/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
struct VM {
    // 4KB (4,096 bytes) of RAM, from location 0x000 (0) to 0xFFF (4095)
    // 0x000 to 0x1FF (512b) reserved for original interpreter, should not be used by programs
    ram: [u8; 4096],
    // 16 general purpose 8-bit registers
    // usually referred to as Vx, where x is a hexadecimal digit (0 through F)
    registers: [u8; 16],
    // register generally used to store memory addresses,
    // so only the lowest (rightmost) 12 bits are usually used.
    i: u16,
    // two special purpose 8-bit registers, for the delay and sound timers
    // when non-zero, automatically decremented at a rate of 60Hz
    dt: u8,
    st: u8, // as long as ST's value is greater than zero, the Chip-8 buzzer will sound
    // program counter (PC), stores the currently executing address
    pc: u16,
    // stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack
    sp: usize,
    // array of 16 u16 values,
    // used to store address that interpreter shoud return to when finished with a subroutine
    // allows 16 levels of nested subroutines
    stack: [u16; 16],
    // 64x32-pixel monochrome display with this format
    // TODO(aalhendi): Render pixels
    display: Screen,
}

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

struct Screen {
    pixels: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            ram: [0; 4096],
            registers: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0,
            sp: 0,
            stack: [0; 16],
            display: Screen::new(),
        }
    }

    /// Clear the display.
    fn cls(&mut self) {
        self.display = Screen::new();
    }

    /// Return from a subroutine.
    /// interpreter sets PC to addr at top of the stack, subtracts 1 from the sp.
    fn ret(&mut self) {
        self.pc = self.stack[self.sp];
        self.sp -= 1;
    }

    /// Jump to a machine code routine at nnn.
    /// This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    fn sys(&mut self, nnn: u16) {
        self.jp(nnn);
    }

    /// Jump to location nnn.
    /// The interpreter sets the program counter to nnn.
    fn jp(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// Call subroutine at nnn.
    /// interpreter increments sp, puts current PC on top of stack. PC is set to nnn.
    fn call(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        self.pc = nnn;
    }

    /// Skip next instruction if Vx = kk.
    /// interpreter compares register Vx to kk, if equal, increments pc by 2.
    fn se_vx_kk(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.pc += 2;
        }
    }

    /// Skip next instruction if Vx != kk.
    /// interpreter compares register Vx to kk, if not equal, increments pc by 2.
    fn sne_vx_kk(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.pc += 2;
        }
    }

    /// Skip next instruction if Vx = Vy.
    /// interpreter compares register Vx to register Vy, if equal, increments pc by 2.
    fn se_vx_vy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
    }

    /// Set Vx = kk.
    /// interpreter puts value kk into register Vx.
    fn ld_vx_kk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    /// Set Vx = Vx + kk.
    /// Adds kk to register Vx, then stores the result in Vx.
    fn add_vx_kk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] += kk;
    }

    /// Set Vx = Vy.
    /// Stores value of register Vy in register Vx.
    fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    }

    /// Set Vx = Vx OR Vy.
    /// Performs bitwise OR on Vx and Vy values, then stores the result in Vx.
    fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    /// Set Vx = Vx AND Vy.
    /// Performs bitwise AND on Vx and Vy values, then stores the result in Vx
    fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    /// Set Vx = Vx XOR Vy.
    /// Performs bitwise XOR on Vx and Vy values, then stores the result in Vx
    fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    /// Set Vx = Vx + Vy, set VF = carry.
    /// Vx and Vy values are added together. If result > 255 (8-bits) Vf set to 1, else 0.
    /// Only lowest 8 bits are kept and stored in Vx
    fn add_vx_vy(&mut self, x: u8, y: u8) {
        let result = (self.registers[x as usize] + self.registers[y as usize]) as usize;
        self.registers[x as usize] = result as u8;
        if result > u8::MAX as usize {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    /// If Vx > Vy, VF set to 1, else 0. Vy is subtracted from Vx, results stored in Vx.
    fn sub_vx_vy(&mut self, x: u8, y: u8) {
        let flag = self.registers[x as usize] > self.registers[y as usize];
        self.registers[x as usize] -= self.registers[y as usize];
        if flag {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    /// Set Vx = Vx >> 1.
    /// If least-significant bit of Vx is 1, VF is set to 1, else 0. Vx is divided by 2.
    fn shr_vx_vy(&mut self, x: u8, _y: u8) {
        let lsb = self.registers[x as usize] & 1;
        self.registers[0xF] = lsb;
        self.registers[x as usize] >>= 1;
    }

    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    /// If Vy > Vx, VF is set to 1, else 0. Vx is subtracted from Vy, results stored in Vx.
    fn subn_vx_vy(&mut self, x: u8, y: u8) {
        let flag = self.registers[y as usize] > self.registers[x as usize];
        self.registers[x as usize] = self.registers[y as usize] - self.registers[x as usize];
        if flag {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    /// Set Vx = Vx << 1.
    /// If most-significant bit of Vx is 1, VF is set to 1, else to 0. Vx is multiplied by 2.
    fn shl_vx_vy(&mut self, x: u8, _y: u8) {
        let msb = (self.registers[x as usize] >> 7) & 1;
        self.registers[0xF] = msb;
        self.registers[x as usize] <<= 1;
    }

    // Skip next instruction if Vx != Vy.
    // Vx and Vy values are compared, if not equal, increments pc by 2.
    fn sne_vx_vy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    }

    // Set I = nnn.
    // The register I value set to nnn.
    fn ld_i_addr(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Jump to location nnn + V0.
    // PC set to nnn plus V0 value.
    fn jp_v0_addr(&mut self, nnn: u16) {
        self.pc = nnn + self.registers[0x0] as u16;
    }

    // Set Vx = random byte AND kk.
    // interpreter generates random number from 0 to 255, ANDed value kk. The results are stored in Vx.
    fn rnd_vx_kk(&mut self, x: u8, kk: u8) {
        // TODO(aalhendi)
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // interpreter reads n bytes from memory, starting at the address stored in I.
    // bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    // Sprites are XORed onto existing screen. If this causes any pixels to be erased, VF is set to 1, else VF set to 0.
    // If sprite is positioned so part is outside the coordinates of the display, it wraps around to opposite side of screen.
    fn drw_vx_vy_n(&mut self, x: u8, y: u8, n: u8) {
        // TODO(aalhendi)
    }

    /// Skip next instruction if key with the value of Vx is pressed.
    /// Checks keyboard, if key equal to the value of Vx is currently in the down position, increments PC by 2.
    fn skp_vx(&mut self, x: u8) {
        // TODO(aalhendi)
    }

    /// Skip next instruction if key with the value of Vx is not pressed.
    /// Checks keyboard, if key equal to the value of Vx is currently in the up position, increments PC by 2.
    fn sknp_vx(&mut self, x: u8) {
        // TODO(aalhendi)
    }

    /// Set Vx = delay timer value.
    /// The value of DT is placed into Vx.
    fn ld_vx_dt(&mut self, x: u8) {
        self.registers[x as usize] = self.dt;
    }

    /// Wait for a key press, store the value of the key in Vx.
    /// All execution stops until a key is pressed, value of that key is stored in Vx.
    fn ld_vx_k(&mut self, x: u8) {
        // TODO(aalhendi)
    }

    /// Set delay timer = Vx.
    /// DT is set equal to the value of Vx.
    fn ld_dt_vx(&mut self, x: u8) {
        self.dt = self.registers[x as usize];
    }

    /// Set sound timer = Vx.
    /// ST is set equal to the value of Vx.
    fn ld_st_vx(&mut self, x: u8) {
        self.st = self.registers[x as usize];
    }

    /// Set I = I + Vx.
    /// I and Vx values are added, results are stored in I.
    fn add_i_vx(&mut self, x: u8) {
        self.i += self.registers[x as usize] as u16;
    }

    /// Set I = location of sprite for digit Vx.
    /// value of I set to location for the hexadecimal sprite equal to the value of Vx.
    fn ld_f_vx(&mut self, x: u8) {
        // TODO(aalhendi)
    }

    /// Store Binary-Coded Decimal (BCD) representation of Vx in memory locations I, I+1, and I+2.
    /// interpreter decimal value of Vx, places (in memory) hundreds digit at location I, tens I+1, ones I+2.
    fn ld_b_vx(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        let hundreds = vx / 100; // Extracts the hundreds digit
        let tens = (vx / 10) % 10; // Extracts the tens digit
        let ones = vx % 10; // Extracts the ones (unit) digit
        let i = self.i as usize;
        self.ram[i] = hundreds;
        self.ram[i + 1] = tens;
        self.ram[i + 2] = ones;
    }

    /// Store registers V0 through Vx in memory starting at location I.
    /// interpreter copies values of registers V0 through Vx into memory, starting at the address in I.
    fn ld_i_vx(&mut self, x: u8) {
        let i = self.i as usize;
        for idx in 0..(x as usize) {
            self.ram[i + idx] = self.registers[idx];
        }
    }

    /// Read registers V0 through Vx from memory starting at location I.
    /// interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn ld_vx_i(&mut self, x: u8) {
        let i = self.i as usize;
        for idx in 0..(x as usize) {
            self.registers[idx] = self.registers[i + idx];
        }
    }

    fn decode(&mut self) {
        let instruction = self.pc;
        let n1 = instruction >> 12; // & 0x000F not needed, shift operation alone aligns the target bits to the rightmost position
        let nnn = instruction & 0x0FFF;
        let n2 = ((instruction >> 8) & 0x000F) as u8;
        let n3 = ((instruction >> 4) & 0x000F) as u8;
        let n4 = (instruction & 0x000F) as u8; // No shift needed, already aligned
        let kk = (instruction & 0x00FF) as u8;
        match n1 {
            0x0 if nnn == 0x00E0 => self.cls(),
            0x0 if nnn == 0x00EE => self.ret(),
            0x0 => self.sys(nnn),
            0x1 => self.jp(nnn),
            0x2 => self.call(nnn),
            0x3 => self.se_vx_kk(n2, kk),
            0x4 => self.sne_vx_kk(n2, kk),
            0x5 if n4 == 0x0 => self.se_vx_vy(n2, n3), // TODO(aalhendi): Check if last n check needed
            0x6 => self.ld_vx_kk(n2, kk),
            0x7 => self.add_vx_kk(n2, kk),
            0x8 if n4 == 0x0 => self.ld_vx_vy(n2, n3),
            0x8 if n4 == 0x1 => self.or_vx_vy(n2, n3),
            0x8 if n4 == 0x2 => self.and_vx_vy(n2, n3),
            0x8 if n4 == 0x3 => self.xor_vx_vy(n2, n3),
            0x8 if n4 == 0x4 => self.add_vx_vy(n2, n3),
            0x8 if n4 == 0x5 => self.sub_vx_vy(n2, n3),
            0x8 if n4 == 0x6 => self.shr_vx_vy(n2, n3),
            0x8 if n4 == 0x7 => self.subn_vx_vy(n2, n3),
            0x8 if n4 == 0xE => self.shl_vx_vy(n2, n3),
            0x9 if n4 == 0x0 => self.sne_vx_vy(n2, n3), // TODO(aalhendi): Check if last n check needed
            0xA => self.ld_i_addr(nnn),
            0xB => self.jp_v0_addr(nnn),
            0xC => self.rnd_vx_kk(n2, kk),
            0xD => self.drw_vx_vy_n(n2, n3, n4),
            0xE if kk == 0x9E => self.skp_vx(n2),
            0xE if kk == 0xA1 => self.sknp_vx(n2),
            0xF if kk == 0x07 => self.ld_vx_dt(n2),
            0xF if kk == 0x0A => self.ld_vx_k(n2),
            0xF if kk == 0x15 => self.ld_dt_vx(n2),
            0xF if kk == 0x18 => self.ld_st_vx(n2),
            0xF if kk == 0x1E => self.add_i_vx(n2),
            0xF if kk == 0x29 => self.ld_f_vx(n2),
            0xF if kk == 0x33 => self.ld_b_vx(n2),
            0xF if kk == 0x55 => self.ld_i_vx(n2),
            0xF if kk == 0x65 => self.ld_vx_i(n2),

            // TODO(aalhendi): Add Super Chip-8 instructions
            _ => unimplemented!(),
        }
    }
}

fn main() {
    let mut vm = VM::new();
    loop {
        vm.decode();
    }
}
