/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
struct VM {
    // 4KB (4,096 bytes) of RAM, from location 0x000 (0) to 0xFFF (4095)
    // 0x000 to 0x1FF (512b) reserved for original interpreter, should not be used by programs
    ram: Vec<u8>,
    // 16 general purpose 8-bit registers
    // usually referred to as Vx, where x is a hexadecimal digit (0 through F)
    registers: Vec<u8>,
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
    sp: u8,
    // array of 16 u16 values,
    // used to store address that interpreter shoud return to when finished with a subroutine
    // allows 16 levels of nested subroutines
    stack: Vec<u16>,
    // 64x32-pixel monochrome display with this format
    // TODO(aalhendi): Render pixels
    pixels: Vec<Vec<bool>>,
}

impl VM {
    // TODO(aalhendi): Add execute function

    // TODO(aalhendi): Add init/new function

    fn decode(&self) -> Instruction {
        let instruction = self.pc;
        let n1 = instruction >> 12; // & 0x000F not needed, shift operation alone aligns the target bits to the rightmost position
        let nnn = instruction & 0x0FFF;
        let n2 = ((instruction >> 8) & 0x000F) as u8;
        let n3 = ((instruction >> 4) & 0x000F) as u8;
        let n4 = (instruction & 0x000F) as u8; // No shift needed, already aligned
        let kk = (instruction & 0x00FF) as u8;
        match n1 {
            0x0 if nnn == 0x00E0 => Instruction::Cls,
            0x0 if nnn == 0x00EE => Instruction::Ret,
            0x0 => Instruction::Sys(nnn),
            0x1 => Instruction::Jp(nnn),
            0x2 => Instruction::Call(nnn),
            0x3 => Instruction::SeXkk(n2, kk),
            0x4 => Instruction::SneXkk(n2, kk),
            0x5 if n4 == 0x0 => Instruction::SeXy(n2, n3), // TODO(aalhendi): Check if last n check needed
            0x6 => Instruction::LdXkk(n2, kk),
            0x7 => Instruction::AddXkk(n2, kk),
            0x8 if n4 == 0x0 => Instruction::LdXy(n2, n3),
            0x8 if n4 == 0x1 => Instruction::OrXy(n2, n3),
            0x8 if n4 == 0x2 => Instruction::AndXy(n2, n3),
            0x8 if n4 == 0x3 => Instruction::XorXy(n2, n3),
            0x8 if n4 == 0x4 => Instruction::AddXy(n2, n3),
            0x8 if n4 == 0x5 => Instruction::SubXy(n2, n3),
            0x8 if n4 == 0x6 => Instruction::ShrXy(n2, n3),
            0x8 if n4 == 0x7 => Instruction::ShlXy(n2, n3),
            0x8 if n4 == 0xE => Instruction::ShlXy(n2, n3),
            0x9 if n4 == 0x0 => Instruction::SneXy(n2, n3), // TODO(aalhendi): Check if last n check needed
            0xA => Instruction::LdIAddr(nnn),
            0xB => Instruction::Jp0Addr(nnn),
            0xC => Instruction::RndXKk(n2, kk),
            0xD => Instruction::DrwXYN(n2, n3, n4),
            0xE if kk == 0x9E => Instruction::SkpX(n2),
            0xE if kk == 0xA1 => Instruction::SknpX(n2),
            0xF if kk == 0x07 => Instruction::LdXDt(n2),
            0xF if kk == 0x0A => Instruction::LdXK(n2),
            0xF if kk == 0x15 => Instruction::LdDtX(n2),
            0xF if kk == 0x18 => Instruction::LdStX(n2),
            0xF if kk == 0x1E => Instruction::AddIX(n2),
            0xF if kk == 0x29 => Instruction::LdFX(n2),
            0xF if kk == 0x33 => Instruction::LdBX(n2),
            0xF if kk == 0x55 => Instruction::LdIX(n2),
            0xF if kk == 0x65 => Instruction::LdXI(n2),

            _ => unimplemented!(),
        }
    }
}

enum Instruction {
    // Jump to a machine code routine at nnn.
    // This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    Sys(u16),
    // Clear the display.
    Cls,
    // Return from a subroutine.
    // interpreter sets PC to addr at top of the stack, subtracts 1 from the sp.
    Ret,
    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    Jp(u16),
    // Call subroutine at nnn.
    // interpreter increments sp, puts current PC on top of stack. PC is set to nnn.
    Call(u16), // addr
    // Skip next instruction if Vx = kk.
    //interpreter compares register Vx to kk, if equal, increments pc by 2.
    SeXkk(u8, u8),
    // Skip next instruction if Vx != kk.
    // interpreter compares register Vx to kk, if not equal, increments pc by 2.
    SneXkk(u8, u8),
    // Skip next instruction if Vx = Vy.
    // interpreter compares register Vx to register Vy, if equal, increments pc by 2.
    SeXy(u8, u8),
    // Set Vx = kk.
    // interpreter puts value kk into register Vx.
    LdXkk(u8, u8),
    // Set Vx = Vx + kk.
    // Adds kk to register Vx, then stores the result in Vx.
    AddXkk(u8, u8),
    // Set Vx = Vy.
    // Stores value of register Vy in register Vx.
    LdXy(u8, u8),
    // Set Vx = Vx OR Vy.
    // Performs bitwise OR on Vx and Vy values, then stores the result in Vx.
    OrXy(u8, u8),
    // Set Vx = Vx AND Vy.
    // Performs bitwise AND on Vx and Vy values, then stores the result in Vx
    AndXy(u8, u8),
    // Set Vx = Vx XOR Vy.
    // Performs bitwise XOR on Vx and Vy values, then stores the result in Vx
    XorXy(u8, u8),
    // Set Vx = Vx + Vy, set VF = carry.
    // Vx and Vy values are added together. If result > 255 (8-bits) Vf set to 1, else 0.
    // Only lowest 8 bits are kept and stored in Vx
    AddXy(u8, u8),
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, VF set to 1, else 0. Vy is subtracted from Vx, results stored in Vx.
    SubXy(u8, u8),
    // Set Vx = Vx >> 1.
    // If least-significant bit of Vx is 1, VF is set to 1, else 0. Vx is divided by 2.
    ShrXy(u8, u8),
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, VF is set to 1, else 0. Vx is subtracted from Vy, results stored in Vx.
    SubnXy(u8, u8),
    // Set Vx = Vx << 1.
    // If most-significant bit of Vx is 1, VF is set to 1, else to 0. Vx is multiplied by 2.
    ShlXy(u8, u8),
    // Skip next instruction if Vx != Vy.
    // Vx and Vy values are compared, if not equal, increments pc by 2.
    SneXy(u8, u8),
    // Set I = nnn.
    // The register I value set to nnn.
    LdIAddr(u16),
    // Jump to location nnn + V0.
    // PC set to nnn plus V0 value.
    Jp0Addr(u16),
    // Set Vx = random byte AND kk.
    // interpreter generates random number from 0 to 255, ANDed value kk. The results are stored in Vx.
    RndXKk(u8, u8),
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // interpreter reads n bytes from memory, starting at the address stored in I.
    // bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    // Sprites are XORed onto existing screen. If this causes any pixels to be erased, VF is set to 1, else VF set to 0.
    // If sprite is positioned so part is outside the coordinates of the display, it wraps around to opposite side of screen.
    DrwXYN(u8, u8, u8),
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks keyboard, if key equal to the value of Vx is currently in the down position, increments PC by 2.
    SkpX(u8),
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks keyboard, if key equal to the value of Vx is currently in the up position, increments PC by 2.
    SknpX(u8),
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.
    LdXDt(u8),
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, value of that key is stored in Vx.
    LdXK(u8),
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    LdDtX(u8),
    // Set sound timer = Vx.
    // ST is set equal to the value of Vx.
    LdStX(u8),
    // Set I = I + Vx.
    // I and Vx values are added, results are stored in I.
    AddIX(u8),
    // Set I = location of sprite for digit Vx.
    // value of I set to location for the hexadecimal sprite equal to the value of Vx.
    LdFX(u8),
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // interpreter decimal value of Vx, places (in memory) hundreds digit at location I, tens I+1, ones I+2.
    LdBX(u8),
    // Store registers V0 through Vx in memory starting at location I.
    // interpreter copies values of registers V0 through Vx into memory, starting at the address in I.
    LdIX(u8),
    // Read registers V0 through Vx from memory starting at location I.
    // interpreter reads values from memory starting at location I into registers V0 through Vx.
    LdXI(u8),
}

fn main() {
    println!("Hello, world!");
}
