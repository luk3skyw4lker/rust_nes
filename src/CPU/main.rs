#![allow(non_snake_case)]
// This is temporary, it will be removed at the end of the project
#![allow(dead_code)]

use std::usize;
pub mod bus;
pub mod instructions;

pub enum Flags {
    C = 1,
    Z = 2,
    I = 4,
    D = 8,
    B = 16,
    U = 32,
    V = 64,
    N = 128,
}

pub struct CPU {
    pc: u16,                      // Program Counter
    a: u8,                        // Register A
    x: u8,                        // Register X
    y: u8,                        // Register Y
    sp: u8,                       // Stack Pointer
    status: u8,                   // Status Register (https://www.nesdev.org/wiki/Status_flags)
    cycles: u8,                   // Cycles Counter
    bus: bus::Bus,                // Memory bus
    instruction_address_abs: u16, // Address for actual instruction absolute
    instruction_address_rel: u16, // Address for actual instruction relative
}

fn is_negative(address: u16) -> bool {
    match address & 0x80 != 0 {
        true => return true,
        false => return false,
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0x0000,
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x00,
            status: 0x00,
            cycles: 0x00,
            bus: bus::Bus::new(),
            instruction_address_abs: 0x0000,
            instruction_address_rel: 0x0000,
        }
    }

    pub fn get_flag(&self, flag: Flags) -> u8 {
        match (self.status & flag as u8) > 0 {
            true => return 1,
            false => return 0,
        };
    }

    pub fn set_flag(&mut self, flag: Flags, value: u8) -> () {
        if value > 0 {
            self.status = self.status | flag as u8;
        } else {
            self.status = self.status & !(flag as u8);
        }
    }

    // Internal Functions
    pub fn reset(&mut self) -> () {
        let hi: u8 = self.bus.read(0xFFFC);
        let low: u8 = self.bus.read(0xFFFD);

        self.pc = u16::from_le_bytes([hi, low]);

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = 0x00 | Flags::U as u8;

        self.cycles = 8;
    }

    pub fn irq(&mut self) -> () {
        if self.get_flag(Flags::I) == 0 {
            self.push((self.pc >> 8) as u8);
            self.push((self.pc & 0x00FF) as u8);

            self.set_flag(Flags::B, 0);
            self.set_flag(Flags::U, 1);
            self.set_flag(Flags::I, 1);

            self.push(self.status as u8);
            self.sp = self.sp.wrapping_sub(1);

            self.pc = self.read_u16_le(0xFFFE);

            self.cycles = 7;
        }
    }

    pub fn nmi(&mut self) -> () {
        self.push((self.pc >> 8) as u8);
        self.push((self.pc & 0x00FF) as u8);

        self.set_flag(Flags::B, 0);
        self.set_flag(Flags::U, 1);
        self.set_flag(Flags::I, 1);

        self.push(self.status as u8);
        self.sp = self.sp.wrapping_sub(1);

        self.pc = self.read_u16_le(0xFFFA);

        self.cycles = 8;
    }

    pub fn step(&mut self) -> () {
        match self.cycles {
            0 => {
                let opcode = self.bus.read(self.pc);

                self.set_flag(Flags::U, 1);
                self.pc = self.pc.wrapping_add(1);

                let instruction = &instructions::Instruction::INSTRUCTIONS[opcode as usize];

                self.cycles = instruction.cycles;

                let extra_mode = (instruction.mode)(self);
                let extra_op = (instruction.function)(self);

                self.cycles = self.cycles.wrapping_add(extra_mode).wrapping_add(extra_op);

                self.cycles = self.cycles.wrapping_sub(1);
            }
            _ => self.cycles = self.cycles.wrapping_sub(1),
        }
    }

    // Helpers
    fn push(&mut self, value: u8) -> () {
        self.bus.write(0x0100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        return self.bus.read(0x0100 + self.sp as u16);
    }

    fn read_u16_le(&mut self, addr: u16) -> u16 {
        let hi = self.bus.read(addr);
        let lo = self.bus.read(addr + 1);

        return u16::from_le_bytes([hi, lo]);
    }

    // Addresssing Modes
    fn IMP(&mut self) -> u8 {
        self.instruction_address_abs = 0;

        return 0;
    }

    fn IMM(&mut self) -> u8 {
        self.instruction_address_abs = self.pc;
        self.pc = self.pc.wrapping_add(1);

        0 as u8
    }

    fn ZP0(&mut self) -> u8 {
        self.instruction_address_abs = self.bus.read(self.pc) as u16 & 0x00FF as u16;

        self.pc = self.pc + 1;

        return 0;
    }

    fn ZPX(&mut self) -> u8 {
        let base = self.bus.read(self.pc) as u16;

        self.pc = self.pc.wrapping_add(1);
        self.instruction_address_abs = (base + self.x as u16) & 0x00FF as u16;

        return 0 as u8;
    }

    fn ZPY(&mut self) -> u8 {
        self.instruction_address_abs = (self.bus.read(self.pc) + self.y) as u16;

        self.pc = self.pc + 1;

        return 0;
    }

    fn REL(&mut self) -> u8 {
        self.instruction_address_rel = (self.bus.read(self.pc)) as u16;

        self.pc = self.pc + 1;

        if is_negative(self.instruction_address_rel) {
            self.instruction_address_rel = (self.instruction_address_rel | 0xFF00) as u16;
        }

        return 0;
    }

    fn ABS(&mut self) -> u8 {
        let hi = self.bus.read(self.pc);
        let lo = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        self.instruction_address_abs = u16::from_le_bytes([hi, lo]);

        return 0;
    }

    fn ABX(&mut self) -> u8 {
        let hi = self.bus.read(self.pc);
        let lo = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        self.instruction_address_abs = u16::from_le_bytes([hi, lo]);
        self.instruction_address_abs = self.instruction_address_abs.wrapping_add(self.x as u16);

        // Check if instruction is changing page and return a additional cycle if true
        if (self.instruction_address_abs & 0xFF00) != u16::from_le_bytes([hi, lo]) {
            return 1;
        }

        return 0;
    }

    fn ABY(&mut self) -> u8 {
        let hi = self.bus.read(self.pc);
        let lo = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        self.instruction_address_abs = u16::from_le_bytes([hi, lo]);
        self.instruction_address_abs = self.instruction_address_abs.wrapping_add(self.y as u16);

        // Check if instruction is changing page and return a additional cycle if true
        if (self.instruction_address_abs & 0xFF00) != u16::from_le_bytes([hi, lo]) {
            return 1;
        }

        return 0;
    }

    // Hardware pointers from 6502
    fn IND(&mut self) -> u8 {
        let ptr_hi = self.bus.read(self.pc);
        let ptr_lo = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        let ptr_full = u16::from_le_bytes([ptr_hi, ptr_lo]);

        if ptr_lo == 0x00FF {
            let hi = self.bus.read(ptr_full & 0xFF00 as u16);
            let lo = self.bus.read(ptr_full + 0);

            self.instruction_address_abs = u16::from_le_bytes([hi, lo]);
        } else {
            let hi = self.bus.read(ptr_full + 1);
            let lo = self.bus.read(ptr_full + 0);

            self.instruction_address_abs = u16::from_le_bytes([hi, lo]);
        }

        return 0;
    }

    fn IZX(&mut self) -> u8 {
        let pointer = self.bus.read(self.pc);

        self.pc = self.pc.wrapping_add(1);

        let hi = self
            .bus
            .read(((pointer as u16) + (self.x as u16)) & 0x00FF as u16);
        let lo = self
            .bus
            .read(((pointer as u16) + (self.x as u16) + 1) & 0x00FF as u16);

        self.instruction_address_abs = u16::from_le_bytes([hi, lo]);

        return 0;
    }

    fn IZY(&mut self) -> u8 {
        let pointer = self.bus.read(self.pc);

        self.pc = self.pc.wrapping_add(1);

        let hi = self.bus.read((pointer as u16) & 0x00FF as u16);
        let lo = self.bus.read(((pointer as u16) + 1) & 0x00FF as u16);

        let base = u16::from_le_bytes([hi, lo]);
        let addr = base.wrapping_add(self.y as u16);

        self.instruction_address_abs = addr;

        if (base & 0xFF00) != (addr & 0xFF00) {
            1
        } else {
            0
        }
    }

    // Instructions
    fn BRK(&mut self) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.set_flag(Flags::I, 1);

        self.bus
            .write(0x0100 + self.sp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.bus
            .write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
        self.sp = self.sp.wrapping_sub(1);

        return 0x00;
    }
    fn ORA(&mut self) -> u8 {
        return 0x00;
    }
    fn ASL(&mut self) -> u8 {
        return 0x00;
    }
    fn PHP(&mut self) -> u8 {
        return 0x00;
    }
    fn BPL(&mut self) -> u8 {
        return 0x00;
    }
    fn CLC(&mut self) -> u8 {
        return 0x00;
    }
    fn JSR(&mut self) -> u8 {
        return 0x00;
    }
    fn AND(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs);

        self.a = self.a & fetched;

        if self.a == 0 {
            self.set_flag(Flags::Z, 1)
        }

        if self.a & 0x80 > 0 {
            self.set_flag(Flags::N, 1);
        }
        return 0x00;
    }
    fn BIT(&mut self) -> u8 {
        return 0x00;
    }
    fn ROL(&mut self) -> u8 {
        return 0x00;
    }
    fn PLP(&mut self) -> u8 {
        return 0x00;
    }
    fn BMI(&mut self) -> u8 {
        return 0x00;
    }
    fn SEC(&mut self) -> u8 {
        return 0x00;
    }
    fn RTI(&mut self) -> u8 {
        return 0x00;
    }
    fn EOR(&mut self) -> u8 {
        return 0x00;
    }
    fn LSR(&mut self) -> u8 {
        return 0x00;
    }
    fn PHA(&mut self) -> u8 {
        return 0x00;
    }
    fn PLA(&mut self) -> u8 {
        return 0x00;
    }
    fn JMP(&mut self) -> u8 {
        return 0x00;
    }
    fn BVC(&mut self) -> u8 {
        return 0x00;
    }
    fn CLI(&mut self) -> u8 {
        return 0x00;
    }
    fn RTS(&mut self) -> u8 {
        return 0x00;
    }
    fn ADC(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        let temp = (self.a + fetched + self.get_flag(Flags::C)) as u128;

        if temp & 0xFF00 > 0 {
            self.set_flag(Flags::C, 1)
        }

        if temp == 0 {
            self.set_flag(Flags::Z, 1)
        }

        if temp & 0x80 > 0 {
            self.set_flag(Flags::N, 1)
        }

        let overflow = (!(self.a as u16 ^ fetched as u16) & (self.a as u16 ^ temp as u16)) & 0x0080;

        if overflow > 0 {
            self.set_flag(Flags::V, 1)
        }

        self.a = (temp & 0x00FF) as u8;

        return 1;
    }
    fn ROR(&mut self) -> u8 {
        return 0x00;
    }
    fn BVS(&mut self) -> u8 {
        return 0x00;
    }
    fn SEI(&mut self) -> u8 {
        return 0x00;
    }
    fn STA(&mut self) -> u8 {
        return 0x00;
    }
    fn STY(&mut self) -> u8 {
        return 0x00;
    }
    fn STX(&mut self) -> u8 {
        return 0x00;
    }
    fn DEY(&mut self) -> u8 {
        return 0x00;
    }
    fn TXA(&mut self) -> u8 {
        return 0x00;
    }
    fn BCC(&mut self) -> u8 {
        return 0x00;
    }
    fn TYA(&mut self) -> u8 {
        return 0x00;
    }
    fn TXS(&mut self) -> u8 {
        return 0x00;
    }
    fn LDY(&mut self) -> u8 {
        return 0x00;
    }
    fn LDA(&mut self) -> u8 {
        return 0x00;
    }
    fn LDX(&mut self) -> u8 {
        return 0x00;
    }
    fn TAY(&mut self) -> u8 {
        return 0x00;
    }
    fn TAX(&mut self) -> u8 {
        return 0x00;
    }
    fn BCS(&mut self) -> u8 {
        return 0x00;
    }
    fn CLV(&mut self) -> u8 {
        return 0x00;
    }
    fn TSX(&mut self) -> u8 {
        return 0x00;
    }
    fn CPY(&mut self) -> u8 {
        return 0x00;
    }
    fn CMP(&mut self) -> u8 {
        return 0x00;
    }
    fn DEC(&mut self) -> u8 {
        return 0x00;
    }
    fn DEX(&mut self) -> u8 {
        return 0x00;
    }
    fn INY(&mut self) -> u8 {
        return 0x00;
    }
    fn BNE(&mut self) -> u8 {
        return 0x00;
    }
    fn CLD(&mut self) -> u8 {
        return 0x00;
    }
    fn CPX(&mut self) -> u8 {
        return 0x00;
    }
    fn SBC(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        let value = (fetched as u16) ^ 0x00FF;

        let temp = (self.a as u16 + value + (self.get_flag(Flags::C) as u16)) as u128;

        if temp & 0xFF00 > 0 {
            self.set_flag(Flags::C, 1)
        }

        if temp == 0 {
            self.set_flag(Flags::Z, 1)
        }

        if temp & 0x80 > 0 {
            self.set_flag(Flags::N, 1)
        }

        let overflow = (!(self.a as u16 ^ fetched as u16) & (self.a as u16 ^ temp as u16)) & 0x0080;

        if overflow > 0 {
            self.set_flag(Flags::V, 1)
        }

        self.a = (temp & 0x00FF) as u8;

        return 1;
    }
    fn INC(&mut self) -> u8 {
        return 0x00;
    }
    fn INX(&mut self) -> u8 {
        return 0x00;
    }
    fn BEQ(&mut self) -> u8 {
        return 0x00;
    }
    fn SED(&mut self) -> u8 {
        return 0x00;
    }
    // illegal opcode function, all illegal opcodes will be mapped to this function
    fn NOP(&mut self) -> u8 {
        return 0x00;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_reads_vector_at_fffc() {
        let mut cpu = CPU::new();

        cpu.bus.write(0xFFFC, 0x00);
        cpu.bus.write(0xFFFD, 0x80);

        cpu.reset();

        assert_eq!(cpu.pc, 0x8000);
    }

    #[test]
    fn irq_pushes_pc_and_status_then_reads_vector_at_fffe() {
        let mut cpu = CPU::new();

        cpu.bus.write(0xFFFE, 0x00);
        cpu.bus.write(0xFFFF, 0x80);

        cpu.irq();

        assert_eq!(cpu.pc, 0x8000);
    }

    #[test]
    fn nmi_pushes_pc_and_status_then_reads_vector_at_fffa() {
        let mut cpu = CPU::new();

        cpu.bus.write(0xFFFA, 0x00);
        cpu.bus.write(0xFFFB, 0x80);

        cpu.nmi();

        assert_eq!(cpu.pc, 0x8000);
    }

    #[test]
    fn step_increments_pc_and_decrements_cycles() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0x00);

        cpu.step();

        assert_eq!(cpu.pc, 0x0001);
    }
}
