#![allow(non_snake_case)]
// This is temporary, it will be removed at the end of the project
#![allow(dead_code)]

use std::usize;
pub mod bus;
pub mod instructions;

pub enum Flags {
    C = 1,   // Carry flag
    Z = 2,   // Zero flag
    I = 4,   // Interrupt flag
    D = 8,   // Decimal flag
    B = 16,  // Break flag
    U = 32,  // Unused flag
    V = 64,  // Overflow flag
    N = 128, // Negative flag
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
    accumulator_mode: bool,       // Whether the instruction operates on the accumulator
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
            accumulator_mode: false,
        }
    }

    pub(crate) fn cycles(&self) -> u8 {
        self.cycles
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
        let lo: u8 = self.bus.read(0xFFFC);
        let hi: u8 = self.bus.read(0xFFFD);

        self.pc = u16::from_le_bytes([lo, hi]);

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
        let lo = self.bus.read(addr);
        let hi = self.bus.read(addr.wrapping_add(1));

        return u16::from_le_bytes([lo, hi]);
    }

    fn set_zn(&mut self, value: u8) {
        self.set_flag(Flags::Z, if value == 0 { 1 } else { 0 });
        self.set_flag(Flags::N, if value & 0x80 > 0 { 1 } else { 0 });
    }

    fn compare(&mut self, register: u8, value: u8) {
        let temp = register.wrapping_sub(value);

        self.set_zn(temp);

        self.set_flag(Flags::C, if register >= value { 1 } else { 0 });
    }

    fn fetch_operand(&mut self) -> u8 {
        if self.accumulator_mode {
            self.a
        } else {
            self.bus.read(self.instruction_address_abs)
        }
    }

    fn store_operand(&mut self, value: u8) {
        if self.accumulator_mode {
            self.a = value;
        } else {
            self.bus.write(self.instruction_address_abs, value);
        }
    }

    fn branch_if(&mut self, condition: bool) -> u8 {
        if !condition {
            return 0;
        }

        let old_pc = self.pc;
        self.pc = self.pc.wrapping_add(self.instruction_address_rel);

        if (old_pc & 0xFF00) != (self.pc & 0xFF00) {
            return 2; // page cross, +1 base taken penalty +1 page = 2 total extra beyond not taken
        }

        return 1; // taken, same page
    }

    fn run_instruction(&mut self) {
        self.cycles = 0;
        self.step();

        while self.cycles > 0 {
            self.step();
        }
    }

    // Addresssing Modes
    fn IMP(&mut self) -> u8 {
        self.accumulator_mode = true;
        self.instruction_address_abs = 0;

        return 0;
    }

    fn IMM(&mut self) -> u8 {
        self.accumulator_mode = false;
        self.instruction_address_abs = self.pc;
        self.pc = self.pc.wrapping_add(1);

        0 as u8
    }

    fn ZP0(&mut self) -> u8 {
        self.accumulator_mode = false;
        let base = self.bus.read(self.pc) as u16;

        self.pc = self.pc.wrapping_add(1);

        self.instruction_address_abs = base & 0x00FF as u16;

        return 0;
    }

    fn ZPX(&mut self) -> u8 {
        self.accumulator_mode = false;
        let base = self.bus.read(self.pc) as u16;

        self.pc = self.pc.wrapping_add(1);

        self.instruction_address_abs = (base + self.x as u16) & 0x00FF as u16;

        return 0 as u8;
    }

    fn ZPY(&mut self) -> u8 {
        self.accumulator_mode = false;
        let base = self.bus.read(self.pc) as u16;

        self.pc = self.pc.wrapping_add(1);

        self.instruction_address_abs = base.wrapping_add(self.y as u16);

        return 0;
    }

    fn REL(&mut self) -> u8 {
        self.accumulator_mode = false;
        self.instruction_address_rel = (self.bus.read(self.pc)) as u16;

        self.pc = self.pc + 1;

        if is_negative(self.instruction_address_rel) {
            self.instruction_address_rel = (self.instruction_address_rel | 0xFF00) as u16;
        }

        return 0;
    }

    fn ABS(&mut self) -> u8 {
        self.accumulator_mode = false;
        let lo = self.bus.read(self.pc);
        let hi = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        self.instruction_address_abs = u16::from_le_bytes([lo, hi]);

        return 0;
    }

    fn ABX(&mut self) -> u8 {
        self.accumulator_mode = false;
        let lo = self.bus.read(self.pc);
        let hi = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        let base = u16::from_le_bytes([lo, hi]);

        self.instruction_address_abs = base.wrapping_add(self.x as u16);

        // Check if instruction is changing page and return a additional cycle if true
        if (self.instruction_address_abs & 0xFF00) != (base & 0xFF00) {
            return 1;
        }

        return 0;
    }

    fn ABY(&mut self) -> u8 {
        self.accumulator_mode = false;
        let lo = self.bus.read(self.pc);
        let hi = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        let base = u16::from_le_bytes([lo, hi]);
        self.instruction_address_abs = base.wrapping_add(self.y as u16);

        // Check if instruction is changing page and return a additional cycle if true
        if (self.instruction_address_abs & 0xFF00) != (base & 0xFF00) {
            return 1;
        }

        return 0;
    }

    // Hardware pointers from 6502
    fn IND(&mut self) -> u8 {
        self.accumulator_mode = false;
        let ptr_lo = self.bus.read(self.pc);
        let ptr_hi = self.bus.read(self.pc.wrapping_add(1));

        self.pc = self.pc.wrapping_add(2);

        let ptr_full = u16::from_le_bytes([ptr_lo, ptr_hi]);

        if ptr_lo == 0x00FF {
            let hi = self.bus.read(ptr_full & 0xFF00 as u16);
            let lo = self.bus.read(ptr_full);

            self.instruction_address_abs = u16::from_le_bytes([lo, hi]);
        } else {
            let hi = self.bus.read(ptr_full.wrapping_add(1));
            let lo = self.bus.read(ptr_full);

            self.instruction_address_abs = u16::from_le_bytes([lo, hi]);
        }

        return 0;
    }

    fn IZX(&mut self) -> u8 {
        self.accumulator_mode = false;
        let pointer = self.bus.read(self.pc);

        self.pc = self.pc.wrapping_add(1);

        let hi = self
            .bus
            .read(((pointer as u16).wrapping_add(self.x as u16)) & 0x00FF as u16);
        let lo = self
            .bus
            .read(((pointer as u16).wrapping_add(self.x as u16).wrapping_add(1)) & 0x00FF as u16);

        self.instruction_address_abs = u16::from_le_bytes([hi, lo]);

        return 0;
    }

    fn IZY(&mut self) -> u8 {
        self.accumulator_mode = false;
        let pointer = self.bus.read(self.pc);

        self.pc = self.pc.wrapping_add(1);

        let hi = self.bus.read((pointer as u16) & 0x00FF as u16);
        let lo = self.bus.read(((pointer as u16) + 1) & 0x00FF as u16);

        let base = u16::from_le_bytes([hi, lo]);

        self.instruction_address_abs = base.wrapping_add(self.y as u16);

        if (base & 0xFF00) != (self.instruction_address_abs & 0xFF00) {
            1
        } else {
            0
        }
    }

    // Instructions
    fn BRK(&mut self) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.push((self.pc >> 8) as u8);
        self.push((self.pc & 0x00FF) as u8);

        self.push(self.status | 0x30);

        self.set_flag(Flags::I, 1);

        self.pc = self.read_u16_le(0xFFFE);

        return 0x00;
    }
    fn ORA(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs);

        self.a = self.a | fetched;

        self.set_zn(self.a);

        return 0x00;
    }
    fn ASL(&mut self) -> u8 {
        let fetched = self.fetch_operand();

        let result = fetched.wrapping_shl(1);

        self.set_flag(Flags::C, if fetched & 0x80 > 0 { 1 } else { 0 });
        self.set_zn(result);

        self.store_operand(result);

        return 0x00;
    }
    fn PHP(&mut self) -> u8 {
        self.push(self.status | 0x30);

        return 0x00;
    }
    fn BPL(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::N) == 0);
    }
    fn CLC(&mut self) -> u8 {
        self.set_flag(Flags::C, 0);

        return 0x00;
    }
    fn JSR(&mut self) -> u8 {
        let return_address = self.pc.wrapping_sub(1);

        self.push(return_address.wrapping_shr(8) as u8);
        self.push((return_address & 0x00FF) as u8);

        self.pc = self.instruction_address_abs;

        return 0x00;
    }
    fn AND(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs);

        self.a = self.a & fetched;

        self.set_zn(self.a);

        return 0x00;
    }
    fn BIT(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs);

        self.set_flag(Flags::Z, if (self.a & fetched) == 0 { 1 } else { 0 });
        self.set_flag(Flags::N, if fetched & 0x80 > 0 { 1 } else { 0 });
        self.set_flag(Flags::V, if fetched & 0x40 > 0 { 1 } else { 0 });

        return 0x00;
    }
    fn ROL(&mut self) -> u8 {
        let fetched = self.fetch_operand();
        let old_c = self.get_flag(Flags::C);

        let result = fetched.wrapping_shl(1) | old_c as u8;

        self.set_flag(Flags::C, if fetched & 0x80 > 0 { 1 } else { 0 });
        self.set_zn(result);

        self.store_operand(result);

        return 0x00;
    }
    fn PLP(&mut self) -> u8 {
        self.status = self.pop() & 0xCF;

        self.set_flag(Flags::U, 1);

        return 0x00;
    }
    fn BMI(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::N) == 1);
    }
    fn SEC(&mut self) -> u8 {
        self.set_flag(Flags::C, 1);

        return 0x00;
    }
    fn RTI(&mut self) -> u8 {
        let status = self.pop();

        let lo = self.pop();
        let hi = self.pop();

        self.pc = u16::from_le_bytes([lo, hi]);

        self.status = status;

        self.set_flag(Flags::B, 0);
        self.set_flag(Flags::U, 1);

        return 0x00;
    }
    fn EOR(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs);

        self.a = self.a ^ fetched;

        self.set_zn(self.a);

        return 0x00;
    }
    fn LSR(&mut self) -> u8 {
        let fetched = self.fetch_operand();

        let result = fetched.wrapping_shr(1);

        self.set_flag(Flags::C, if fetched & 0x01 > 0 { 1 } else { 0 });
        self.set_zn(result);

        self.store_operand(result);

        return 0x00;
    }
    fn PHA(&mut self) -> u8 {
        self.push(self.a);
        return 0x00;
    }
    fn PLA(&mut self) -> u8 {
        self.a = self.pop();
        self.set_zn(self.a);

        return 0x00;
    }
    fn JMP(&mut self) -> u8 {
        self.pc = self.instruction_address_abs;

        return 0x00;
    }
    fn BVC(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::V) == 0);
    }
    fn CLI(&mut self) -> u8 {
        self.set_flag(Flags::I, 0);

        return 0x00;
    }
    fn RTS(&mut self) -> u8 {
        let lo = self.pop();
        let hi = self.pop();

        let addr = u16::from_le_bytes([lo, hi]);

        self.pc = addr.wrapping_add(1);

        return 0x00;
    }
    fn ADC(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        let temp = (self.a as u16) + (fetched as u16) + (self.get_flag(Flags::C) as u16);

        self.set_flag(Flags::C, if temp > 0xFF { 1 } else { 0 });
        self.set_flag(
            Flags::V,
            if ((!(self.a as u16 ^ fetched as u16) & (self.a as u16 ^ temp as u16)) & 0x0080) > 0 {
                1
            } else {
                0
            },
        );

        self.a = temp as u8;
        self.set_zn(self.a);

        return 0x00;
    }
    fn ROR(&mut self) -> u8 {
        let fetched = self.fetch_operand();
        let old_c = self.get_flag(Flags::C);

        let result = fetched.wrapping_shr(1) | (old_c as u8) << 7;

        self.set_flag(Flags::C, if fetched & 0x01 > 0 { 1 } else { 0 });
        self.set_zn(result);

        self.store_operand(result);

        return 0x00;
    }
    fn BVS(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::V) == 1);
    }
    fn SEI(&mut self) -> u8 {
        self.set_flag(Flags::I, 1);

        return 0x00;
    }
    fn STA(&mut self) -> u8 {
        self.bus.write(self.instruction_address_abs as u16, self.a);

        return 0x00;
    }
    fn STY(&mut self) -> u8 {
        self.bus.write(self.instruction_address_abs as u16, self.y);

        return 0x00;
    }
    fn STX(&mut self) -> u8 {
        self.bus.write(self.instruction_address_abs as u16, self.x);

        return 0x00;
    }
    fn DEY(&mut self) -> u8 {
        self.y = self.y.wrapping_sub(1);
        self.set_zn(self.y);

        return 0x00;
    }
    fn TXA(&mut self) -> u8 {
        self.a = self.x;
        self.set_zn(self.a);

        return 0x00;
    }
    fn BCC(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::C) == 0);
    }
    fn TYA(&mut self) -> u8 {
        self.a = self.y;
        self.set_zn(self.a);

        return 0x00;
    }
    fn TXS(&mut self) -> u8 {
        self.sp = self.x;

        return 0x00;
    }
    fn LDY(&mut self) -> u8 {
        self.y = self.bus.read(self.instruction_address_abs as u16);
        self.set_zn(self.y);

        return 0x00;
    }
    fn LDA(&mut self) -> u8 {
        self.a = self.bus.read(self.instruction_address_abs as u16);
        self.set_zn(self.a);

        return 0x00;
    }
    fn LDX(&mut self) -> u8 {
        self.x = self.bus.read(self.instruction_address_abs as u16);
        self.set_zn(self.x);

        return 0x00;
    }
    fn TAY(&mut self) -> u8 {
        self.y = self.a;
        self.set_zn(self.y);

        return 0x00;
    }
    fn TAX(&mut self) -> u8 {
        self.x = self.a;
        self.set_zn(self.x);

        return 0x00;
    }
    fn BCS(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::C) == 1);
    }
    fn CLV(&mut self) -> u8 {
        self.set_flag(Flags::V, 0);

        return 0x00;
    }
    fn TSX(&mut self) -> u8 {
        self.x = self.sp;
        self.set_zn(self.x);

        return 0x00;
    }
    fn CPY(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        self.compare(self.y, fetched);

        return 0x00;
    }
    fn CMP(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        self.compare(self.a, fetched);

        return 0x00;
    }
    fn DEC(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);
        let temp = fetched.wrapping_sub(1);

        self.bus.write(self.instruction_address_abs as u16, temp);
        self.set_zn(temp);

        return 0x00;
    }
    fn DEX(&mut self) -> u8 {
        self.x = self.x.wrapping_sub(1);
        self.set_zn(self.x);

        return 0x00;
    }
    fn INY(&mut self) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.set_zn(self.y);

        return 0x00;
    }
    fn BNE(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::Z) == 0);
    }
    fn CLD(&mut self) -> u8 {
        self.set_flag(Flags::D, 0);

        return 0x00;
    }
    fn CPX(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        self.compare(self.x, fetched);

        return 0x00;
    }
    fn SBC(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);

        let value = (fetched as u16) ^ 0x00FF;

        let temp = (self.a as u16 + value + (self.get_flag(Flags::C) as u16)) as u16;

        self.set_flag(Flags::C, if temp > 0xFF { 1 } else { 0 });
        self.set_flag(
            Flags::V,
            if ((!(self.a as u16 ^ fetched as u16) & (self.a as u16 ^ temp as u16)) & 0x0080) > 0 {
                1
            } else {
                0
            },
        );

        self.a = (temp & 0x00FF) as u8;
        self.set_zn(self.a);

        return 0x00;
    }
    fn INC(&mut self) -> u8 {
        let fetched = self.bus.read(self.instruction_address_abs as u16);
        let temp = fetched.wrapping_add(1);

        self.bus.write(self.instruction_address_abs as u16, temp);
        self.set_zn(temp);

        return 0x00;
    }

    fn INX(&mut self) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.set_zn(self.x);

        return 0x00;
    }

    fn BEQ(&mut self) -> u8 {
        return self.branch_if(self.get_flag(Flags::Z) == 1);
    }

    fn SED(&mut self) -> u8 {
        self.set_flag(Flags::D, 1);

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

        cpu.bus.write(0x0000, 0xEA); // NOP

        cpu.step();

        assert_eq!(cpu.pc, 0x0001);
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn imm_resolves_to_correct_address() {
        let mut cpu = CPU::new();

        cpu.pc = 1; // after opcode fetch

        cpu.bus.write(0x0001, 0x42);
        cpu.IMM();

        assert_eq!(cpu.instruction_address_abs, 0x0001);
        assert_eq!(cpu.pc, 0x0002);
    }

    #[test]
    fn set_zn_clears_and_sets_flags() {
        let mut cpu = CPU::new();

        cpu.set_zn(0x00);

        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);

        cpu.set_zn(0x80);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);
    }

    #[test]
    fn lda_immediate_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x00, 0xA9);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0x00;
        cpu.cycles = 0;

        cpu.run_instruction();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn ldx_immediate_mode_sets_x_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x00, 0xA2);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0x00;
        cpu.cycles = 0;

        cpu.run_instruction();

        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn ldy_immediate_mode_sets_y_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x00, 0xA0);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0x00;
        cpu.cycles = 0;

        cpu.run_instruction();

        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn sta_absolute_mode_writes_a_to_memory() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x00, 0x85);
        cpu.bus.write(0x01, 0x10);

        cpu.run_instruction();

        assert_eq!(cpu.bus.read(0x0010), 0x42);
    }

    #[test]
    fn stx_absolute_mode_writes_x_to_memory() {
        let mut cpu = CPU::new();

        cpu.x = 0x42;
        cpu.bus.write(0x00, 0x86);
        cpu.bus.write(0x01, 0x10);

        cpu.run_instruction();

        assert_eq!(cpu.bus.read(0x0010), 0x42);
    }

    #[test]
    fn sty_absolute_mode_writes_y_to_memory() {
        let mut cpu = CPU::new();

        cpu.y = 0x42;
        cpu.bus.write(0x00, 0x84);
        cpu.bus.write(0x01, 0x10);

        cpu.run_instruction();

        assert_eq!(cpu.bus.read(0x0010), 0x42);
    }

    #[test]
    fn txa_sets_a_to_x_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.x = 0x42;

        cpu.bus.write(0x00, 0x8A);
        cpu.run_instruction();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn tya_sets_a_to_y_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.y = 0x42;

        cpu.bus.write(0x00, 0x98);
        cpu.run_instruction();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn tax_sets_a_to_x_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;

        cpu.bus.write(0x00, 0xAA);
        cpu.run_instruction();

        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn tay_sets_a_to_y_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;

        cpu.bus.write(0x00, 0xA8);
        cpu.run_instruction();

        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn tsx_sets_sp_to_x_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.sp = 0x42;

        cpu.bus.write(0x00, 0xBA);
        cpu.run_instruction();

        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn txs_sets_sp_to_x() {
        let mut cpu = CPU::new();

        cpu.x = 0x42;

        cpu.bus.write(0x00, 0x9A);
        cpu.run_instruction();

        assert_eq!(cpu.sp, 0x42);
    }

    #[test]
    fn php_plp_round_trip() {
        let mut cpu = CPU::new();
        cpu.sp = 0xFD;

        cpu.set_flag(Flags::C, 1);
        cpu.set_flag(Flags::N, 1);
        cpu.set_flag(Flags::V, 1);

        cpu.bus.write(0x00, 0x08); // PHP
        cpu.bus.write(0x01, 0x28); // PLP
        cpu.pc = 0;

        cpu.run_instruction(); // PHP
        cpu.run_instruction(); // PLP

        assert_eq!(cpu.get_flag(Flags::C), 1);
        assert_eq!(cpu.get_flag(Flags::N), 1);
        assert_eq!(cpu.get_flag(Flags::V), 1);
        assert_eq!(cpu.get_flag(Flags::U), 1);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn pha_pla_round_trip_works() {
        let mut cpu = CPU::new();

        cpu.a = 0xDE;
        cpu.sp = 0xFD;

        cpu.bus.write(0x00, 0x48);
        cpu.bus.write(0x01, 0x68);
        cpu.pc = 0;

        cpu.run_instruction(); // PHA
        cpu.a = 0x00; // smash A so PLA must restore it
        cpu.run_instruction(); // PLA

        assert_eq!(cpu.a, 0xDE);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn ora_immediate_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x00;
        cpu.bus.write(0x00, 0x09);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // ORA

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn and_immediate_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0xFF;
        cpu.bus.write(0x00, 0x29);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // AND

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn eor_immediate_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x00;
        cpu.bus.write(0x00, 0x49);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // EOR

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn adc_immediate_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x00;
        cpu.bus.write(0x00, 0x69);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // ADC

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::C), 0);
    }

    #[test]
    fn sbc_immediate_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.set_flag(Flags::C, 1);
        cpu.bus.write(0x00, 0xE9);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // SBC

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::C), 1);
    }

    #[test]
    fn cmp_compares_a_and_value_and_sets_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x00, 0xC9);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // CMP

        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::V), 0);
    }

    #[test]
    fn cpy_compares_y_and_value_and_sets_flags() {
        let mut cpu = CPU::new();

        cpu.y = 0x42;
        cpu.bus.write(0x00, 0xC0);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // CPY

        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::V), 0);
    }

    #[test]
    fn cpx_compares_x_and_value_and_sets_flags() {
        let mut cpu = CPU::new();

        cpu.x = 0x42;
        cpu.bus.write(0x00, 0xE0);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // CPX

        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::V), 0);
    }

    #[test]
    fn bit_sets_flags_based_on_value() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x00, 0x24);
        cpu.bus.write(0x01, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // BIT

        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::V), 0);
    }

    #[test]
    fn iny_increments_y_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.y = 0x42;
        cpu.bus.write(0x00, 0xC8);
        cpu.pc = 0;

        cpu.run_instruction(); // INY

        assert_eq!(cpu.y, 0x43);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn dex_decrements_x_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.x = 0x42;
        cpu.bus.write(0x00, 0xCA);
        cpu.pc = 0;

        cpu.run_instruction(); // DEX

        assert_eq!(cpu.x, 0x41);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn inx_increments_x_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.x = 0x42;
        cpu.bus.write(0x00, 0xE8);
        cpu.pc = 0;

        cpu.run_instruction(); // INX

        assert_eq!(cpu.x, 0x43);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn dey_decrements_y_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.y = 0x42;
        cpu.bus.write(0x00, 0x88);
        cpu.pc = 0;

        cpu.run_instruction(); // DEY

        assert_eq!(cpu.y, 0x41);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn inc_increments_value_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0042, 0x42); // value at $42
        cpu.bus.write(0x0000, 0xE6); // INC zp
        cpu.bus.write(0x0001, 0x42); // address = $42
        cpu.pc = 0;

        cpu.run_instruction(); // INC

        assert_eq!(cpu.bus.read(0x0042), 0x43);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn dec_decrements_value_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0042, 0x42); // value at $42
        cpu.bus.write(0x0000, 0xC6); // DEC zp
        cpu.bus.write(0x0001, 0x42); // address = $42
        cpu.pc = 0;

        cpu.run_instruction(); // DEC

        assert_eq!(cpu.bus.read(0x0042), 0x41);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn asl_accumulator_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x0000, 0x0A);
        cpu.pc = 0;

        cpu.run_instruction(); // ASL

        assert_eq!(cpu.a, 0x84);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);
        assert_eq!(cpu.bus.read(0x0000), 0x0A);
    }

    #[test]
    fn lsr_accumulator_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x0000, 0x4A);
        cpu.pc = 0;

        cpu.run_instruction(); // LSR

        assert_eq!(cpu.a, 0x21);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn rol_accumulator_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x0000, 0x2A);
        cpu.pc = 0;

        cpu.run_instruction(); // ROL

        assert_eq!(cpu.a, 0x84);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);

        // Test with carry flag
        cpu.a = 0x01;
        cpu.set_flag(Flags::C, 1);
        cpu.bus.write(0x0000, 0x2A);
        cpu.pc = 0;

        cpu.run_instruction(); // ROL

        assert_eq!(cpu.a, 0x03);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::C), 0);
    }

    #[test]
    fn ror_accumulator_mode_sets_a_and_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x42;
        cpu.bus.write(0x0000, 0x6A);
        cpu.pc = 0;

        cpu.run_instruction(); // ROR

        assert_eq!(cpu.a, 0x21);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);

        // Test with carry flag
        cpu.a = 0x00;
        cpu.set_flag(Flags::C, 1);
        cpu.bus.write(0x0000, 0x6A);
        cpu.pc = 0;

        cpu.run_instruction(); // ROR

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);
        assert_eq!(cpu.get_flag(Flags::C), 0);
    }

    #[test]
    fn asl_non_accumulator_mode_sets_zn_flags() {
        let mut cpu = CPU::new();

        cpu.a = 0x00;
        cpu.bus.write(0x0042, 0x42);
        cpu.bus.write(0x0000, 0x0E);
        cpu.bus.write(0x0001, 0x42);
        cpu.bus.write(0x0002, 0x00);
        cpu.pc = 0;

        cpu.run_instruction(); // ASL

        assert_eq!(cpu.bus.read(0x0042), 0x84);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);
        assert_eq!(cpu.get_flag(Flags::C), 0);
        assert_eq!(cpu.a, 0x00);

        // Test with carry flag
        cpu.bus.write(0x0042, 0x80);
        cpu.bus.write(0x0000, 0x0E);
        cpu.bus.write(0x0001, 0x42);
        cpu.bus.write(0x0002, 0x00);
        cpu.pc = 0;

        cpu.run_instruction(); // ASL

        assert_eq!(cpu.bus.read(0x0042), 0x00);
        assert_eq!(cpu.get_flag(Flags::Z), 1);
        assert_eq!(cpu.get_flag(Flags::N), 0);
        assert_eq!(cpu.get_flag(Flags::C), 1);
    }

    #[test]
    fn lsr_non_accumulator_mode_sets_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0042, 0x42);
        cpu.bus.write(0x0000, 0x4E);
        cpu.bus.write(0x0001, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // LSR

        assert_eq!(cpu.bus.read(0x0042), 0x21);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn rol_non_accumulator_mode_sets_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0042, 0x42);
        cpu.bus.write(0x0000, 0x2E);
        cpu.bus.write(0x0001, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // ROL

        assert_eq!(cpu.bus.read(0x0042), 0x84);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 1);
    }

    #[test]
    fn ror_non_accumulator_mode_sets_zn_flags() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0042, 0x42);
        cpu.bus.write(0x0000, 0x6E);
        cpu.bus.write(0x0001, 0x42);
        cpu.pc = 0;

        cpu.run_instruction(); // ROR

        assert_eq!(cpu.bus.read(0x0042), 0x21);
        assert_eq!(cpu.get_flag(Flags::Z), 0);
        assert_eq!(cpu.get_flag(Flags::N), 0);
    }

    #[test]
    fn jmp_absolute_sets_pc() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0x4C);
        cpu.bus.write(0x0001, 0x42);
        cpu.bus.write(0x0002, 0x00);
        cpu.pc = 0;

        cpu.run_instruction(); // JMP

        assert_eq!(cpu.pc, 0x0042);
    }

    #[test]
    fn jmp_absolute_indirect_sets_pc() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0042, 0x34); // lo
        cpu.bus.write(0x0043, 0x12); // hi
        cpu.bus.write(0x0000, 0x6C); // JMP ($0042)
        cpu.bus.write(0x0001, 0x42);
        cpu.bus.write(0x0002, 0x00);
        cpu.pc = 0;

        cpu.run_instruction();

        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn jsr_absolute_sets_pc() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0x20);
        cpu.bus.write(0x0001, 0x42);
        cpu.bus.write(0x0002, 0x00);

        cpu.run_instruction(); // JSR

        assert_eq!(cpu.pc, 0x0042);
    }

    #[test]
    fn jsr_rts_round_trip_test() {
        let mut cpu = CPU::new();

        cpu.sp = 0xFD;

        cpu.bus.write(0x0000, 0x20); // JSR
        cpu.bus.write(0x0001, 0x10);
        cpu.bus.write(0x0002, 0x00);
        cpu.bus.write(0x0003, 0xA9); // LDA #$42
        cpu.bus.write(0x0004, 0x42);
        cpu.bus.write(0x0010, 0x60); // RTS

        cpu.pc = 0;
        cpu.run_instruction(); // JSR
        assert_eq!(cpu.pc, 0x0010);

        cpu.run_instruction(); // RTS
        assert_eq!(cpu.pc, 0x0003);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.sp, 0xFD); // stack restored
    }

    #[test]
    fn rti_round_trip_test() {
        let mut cpu = CPU::new();

        cpu.sp = 0xFD;
        cpu.set_flag(Flags::C, 1);
        cpu.set_flag(Flags::N, 1);

        cpu.push(0x12);
        cpu.push(0x34);
        cpu.push(cpu.status | 0x20);

        cpu.bus.write(0x000, 0x40);
        cpu.pc = 0;
        cpu.status = 0;

        cpu.run_instruction(); // RTI

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.get_flag(Flags::C), 1);
        assert_eq!(cpu.get_flag(Flags::N), 1);
        assert_eq!(cpu.get_flag(Flags::U), 1);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn beq_branches_if_zero_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9);
        cpu.bus.write(0x0001, 0x00);
        cpu.bus.write(0x0002, 0xF0);
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9);
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA

        assert_eq!(cpu.get_flag(Flags::Z), 1);

        cpu.run_instruction(); // BEQ

        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bne_branches_if_zero_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9);
        cpu.bus.write(0x0001, 0xFF);
        cpu.bus.write(0x0002, 0xD0);
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9);

        cpu.run_instruction(); // LDA

        assert_eq!(cpu.get_flag(Flags::Z), 0);

        cpu.run_instruction(); // BNE

        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bpl_branches_if_negative_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9);
        cpu.bus.write(0x0001, 0x01);
        cpu.bus.write(0x0002, 0x10);
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9);
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA

        assert_eq!(cpu.get_flag(Flags::N), 0);

        cpu.run_instruction(); // BPL

        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bvc_branches_if_overflow_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9);
        cpu.bus.write(0x0001, 0x00);
        cpu.bus.write(0x0002, 0x50);
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9);
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA

        assert_eq!(cpu.get_flag(Flags::V), 0);

        cpu.run_instruction(); // BVC

        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bcc_branches_if_carry_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9);
        cpu.bus.write(0x0001, 0x00);
        cpu.bus.write(0x0002, 0x90);
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9);

        cpu.run_instruction(); // LDA

        assert_eq!(cpu.get_flag(Flags::C), 0);

        cpu.run_instruction(); // BCC

        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bmi_branches_if_negative_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$80
        cpu.bus.write(0x0001, 0x80);
        cpu.bus.write(0x0002, 0x30); // BMI +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // skipped if branch taken
        cpu.bus.write(0x0005, 0xFF); // LDA #$FF

        cpu.run_instruction(); // LDA

        assert_eq!(cpu.get_flag(Flags::N), 1);

        cpu.run_instruction(); // BMI

        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bvs_branches_if_overflow_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$50
        cpu.bus.write(0x0001, 0x50);
        cpu.bus.write(0x0002, 0x69); // ADC #$50
        cpu.bus.write(0x0003, 0x50);
        cpu.bus.write(0x0004, 0x70); // BVS +2
        cpu.bus.write(0x0005, 0x02);
        cpu.bus.write(0x0006, 0xA9); // skipped if branch taken
        cpu.bus.write(0x0007, 0xFF);

        cpu.run_instruction(); // LDA
        cpu.run_instruction(); // ADC

        assert_eq!(cpu.get_flag(Flags::V), 1);

        cpu.run_instruction(); // BVS

        assert_eq!(cpu.pc, 0x0008);
    }

    #[test]
    fn bcs_branches_if_carry_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$FF
        cpu.bus.write(0x0001, 0xFF);
        cpu.bus.write(0x0002, 0x69); // ADC #$01 (carry set)
        cpu.bus.write(0x0003, 0x01);
        cpu.bus.write(0x0004, 0xB0); // BCS +2
        cpu.bus.write(0x0005, 0x02);
        cpu.bus.write(0x0006, 0xA9); // skipped if branch taken
        cpu.bus.write(0x0007, 0xFF);

        cpu.run_instruction(); // LDA
        cpu.run_instruction(); // ADC

        assert_eq!(cpu.get_flag(Flags::C), 1);

        cpu.run_instruction(); // BCS

        assert_eq!(cpu.pc, 0x0008);
    }

    #[test]
    fn beq_does_not_branch_if_zero_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$01
        cpu.bus.write(0x0001, 0x01);
        cpu.bus.write(0x0002, 0xF0); // BEQ +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // fall-through
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.get_flag(Flags::Z), 0);

        cpu.run_instruction(); // BEQ
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn bne_does_not_branch_if_zero_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$00
        cpu.bus.write(0x0001, 0x00);
        cpu.bus.write(0x0002, 0xD0); // BNE +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // fall-through
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.get_flag(Flags::Z), 1);

        cpu.run_instruction(); // BNE
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn bpl_does_not_branch_if_negative_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$80
        cpu.bus.write(0x0001, 0x80);
        cpu.bus.write(0x0002, 0x10); // BPL +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // fall-through
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.get_flag(Flags::N), 1);

        cpu.run_instruction(); // BPL
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn bmi_does_not_branch_if_negative_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$01
        cpu.bus.write(0x0001, 0x01);
        cpu.bus.write(0x0002, 0x30); // BMI +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // fall-through
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.get_flag(Flags::N), 0);

        cpu.run_instruction(); // BMI
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn bvc_does_not_branch_if_overflow_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$50
        cpu.bus.write(0x0001, 0x50);
        cpu.bus.write(0x0002, 0x69); // ADC #$50 → V=1
        cpu.bus.write(0x0003, 0x50);
        cpu.bus.write(0x0004, 0x50); // BVC +2
        cpu.bus.write(0x0005, 0x02);
        cpu.bus.write(0x0006, 0xA9); // fall-through
        cpu.bus.write(0x0007, 0xFF);

        cpu.run_instruction(); // LDA
        cpu.run_instruction(); // ADC
        assert_eq!(cpu.get_flag(Flags::V), 1);

        cpu.run_instruction(); // BVC
        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bvs_does_not_branch_if_overflow_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$00
        cpu.bus.write(0x0001, 0x00);
        cpu.bus.write(0x0002, 0x70); // BVS +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // fall-through
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.get_flag(Flags::V), 0);

        cpu.run_instruction(); // BVS
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn bcc_does_not_branch_if_carry_flag_is_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$FF
        cpu.bus.write(0x0001, 0xFF);
        cpu.bus.write(0x0002, 0x69); // ADC #$01 → C=1
        cpu.bus.write(0x0003, 0x01);
        cpu.bus.write(0x0004, 0x90); // BCC +2
        cpu.bus.write(0x0005, 0x02);
        cpu.bus.write(0x0006, 0xA9); // fall-through
        cpu.bus.write(0x0007, 0xFF);

        cpu.run_instruction(); // LDA
        cpu.run_instruction(); // ADC
        assert_eq!(cpu.get_flag(Flags::C), 1);

        cpu.run_instruction(); // BCC
        assert_eq!(cpu.pc, 0x0006);
    }

    #[test]
    fn bcs_does_not_branch_if_carry_flag_is_not_set() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x0000, 0xA9); // LDA #$00
        cpu.bus.write(0x0001, 0x00);
        cpu.bus.write(0x0002, 0xB0); // BCS +2
        cpu.bus.write(0x0003, 0x02);
        cpu.bus.write(0x0004, 0xA9); // fall-through
        cpu.bus.write(0x0005, 0xFF);

        cpu.run_instruction(); // LDA
        assert_eq!(cpu.get_flag(Flags::C), 0);

        cpu.run_instruction(); // BCS
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn page_cross_boundary_branch_works() {
        let mut cpu = CPU::new();

        cpu.bus.write(0x00FB, 0xA9); // LDA #$01
        cpu.bus.write(0x00FC, 0x01);
        cpu.bus.write(0x00FD, 0xD0); // BNE +1
        cpu.bus.write(0x00FE, 0x01);

        cpu.pc = 0x00FB;
        cpu.run_instruction(); // LDA
        cpu.step();

        assert_eq!(cpu.pc, 0x0100);
        assert_eq!(cpu.cycles(), 3);
    }

    #[test]
    fn brk_interrupts_program_counter() {
        let mut cpu = CPU::new();
        cpu.sp = 0xFD;

        cpu.bus.write(0xFFFE, 0x00);
        cpu.bus.write(0xFFFF, 0x80);

        cpu.bus.write(0x0000, 0x00); // BRK
        cpu.bus.write(0x0001, 0x00); // padding
        cpu.pc = 0;

        cpu.run_instruction(); // BRK

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0xFA);
        assert_eq!(cpu.bus.read(0x01FD), 0x00); // PC hi byte
        assert_eq!(cpu.bus.read(0x01FC), 0x02); // PC lo byte
        assert_eq!(cpu.bus.read(0x01FB) & 0x10, 0x10); // B flag set
        assert_eq!(cpu.get_flag(Flags::I), 1);
    }
}
