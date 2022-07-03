#![allow(non_snake_case)]
// This is temporary, it will be removed at the end of the project
#![allow(dead_code)]
pub mod bus;
pub mod instructions;

enum Flags {
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

fn isNegative(address: u16) -> bool {
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

	pub fn GetFlag(&self, flag: u8) -> u8 {
		match (self.status & flag) > 0 {
			true => return 1,
			false => return 0,
		};
	}

	pub fn SetFlag(&mut self, flag: u8, value: u8) -> () {
		if flag > 0 {
			self.status = self.status | value;
		} else {
			self.status = self.status & !value;
		}
	}

	// Internal Functions
	pub fn reset(&mut self) -> () {
		let hi: u8 = self.bus.read(0xFFCC);
		let low: u8 = self.bus.read(0xFFCD);

		self.pc = ((hi as u16) << 8) | low as u16;

		self.a = 0;
		self.x = 0;
		self.y = 0;
		self.sp = 0xFD;
		self.status = 0x00 | Flags::U as u8;

		self.cycles = 8;
	}
	pub fn irq(&mut self) -> () {
		if self.GetFlag(1 << 2) == 0 {
			self
				.bus
				.write(0x0100 + self.sp as u16, ((self.pc >> 8) & 0x00FF) as u8);
			self.sp = self.sp - 1;
			self
				.bus
				.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
			self.sp = self.sp - 1;

			self.SetFlag(Flags::B as u8, 0);
			self.SetFlag(Flags::U as u8, 1);
			self.SetFlag(Flags::I as u8, 1);

			self.bus.write(0x1000 + self.sp as u16, self.status);
			self.sp = self.sp - 1;

			let hi: u8 = self.bus.read(0xFFFE);
			let lo: u8 = self.bus.read(0xFFFF);

			self.pc = ((hi as u16) << 8) | lo as u16;

			self.cycles = 7;
		}
	}
	pub fn nmi(&mut self) -> () {
		self
			.bus
			.write(0x0100 + self.sp as u16, ((self.pc >> 8) & 0x00FF) as u8);
		self.sp = self.sp - 1;
		self
			.bus
			.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
		self.sp = self.sp - 1;

		self.SetFlag(Flags::B as u8, 0);
		self.SetFlag(Flags::U as u8, 1);
		self.SetFlag(Flags::I as u8, 1);

		self.bus.write(0x1000 + self.sp as u16, self.status);
		self.sp = self.sp - 1;

		let hi: u8 = self.bus.read(0xFFFA);
		let lo: u8 = self.bus.read(0xFFFB);

		self.pc = ((hi as u16) << 8) | lo as u16;

		self.cycles = 8;
	}
	pub fn step(&mut self) -> () {
		match self.cycles {
			0 => {
				let opcode = self.bus.read(self.pc);

				self.SetFlag(Flags::U as u8, 1);
				self.pc = self.pc + 1;

				let instruction = &instructions::Instruction::INSTRUCTIONS[opcode as usize];

				self.cycles = instruction.cycles;

				let more_cycles1 = (instruction.function)(self);
				let more_cycles2 = (instruction.mode)(self);

				self.cycles = self.cycles + more_cycles1 + more_cycles2;

				self.SetFlag(Flags::U as u8, 1);

				self.cycles = self.cycles - 1;
			}
			_ => self.cycles = self.cycles - 1,
		}
	}

	// Addresssing Modes
	fn IMP(&mut self) -> u8 {
		self.instruction_address_abs = 0;

		return 0;
	}

	fn IMM(&mut self) -> u8 {
		self.instruction_address_abs = self.pc + 1;

		return 0;
	}

	fn ZP0(&mut self) -> u8 {
		self.instruction_address_abs = self.bus.read(self.pc) as u16 & 0x00FF as u16;

		self.pc = self.pc + 1;

		return 0;
	}

	fn ZPX(&mut self) -> u8 {
		self.instruction_address_abs = (self.bus.read(self.pc) + self.x) as u16 & 0x00FF as u16;

		self.pc = self.pc + 1;

		return 0;
	}

	fn ZPY(&mut self) -> u8 {
		self.instruction_address_abs = (self.bus.read(self.pc) + self.y) as u16;

		self.pc = self.pc + 1;

		return 0;
	}

	fn REL(&mut self) -> u8 {
		self.instruction_address_rel = (self.bus.read(self.pc)) as u16;

		self.pc = self.pc + 1;

		if isNegative(self.instruction_address_rel) {
			self.instruction_address_rel = (self.instruction_address_rel | 0xFF00) as u16;
		}

		return 0;
	}

	fn ABS(&mut self) -> u8 {
		let hi = self.bus.read(self.pc);
		let lo = self.bus.read(self.pc + 1);

		self.pc = self.pc + 2;

		self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;

		return 0;
	}

	fn ABX(&mut self) -> u8 {
		let hi = self.bus.read(self.pc);
		let lo = self.bus.read(self.pc + 1);

		self.pc = self.pc + 2;

		self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;
		self.instruction_address_abs = self.instruction_address_abs + self.x as u16;

		// Check if instruction is changing page and return a additional cycle if true
		if (self.instruction_address_abs & 0xFF00) != ((hi as u16) << 8) {
			return 1;
		}

		return 0;
	}

	fn ABY(&mut self) -> u8 {
		let hi = self.bus.read(self.pc);
		let lo = self.bus.read(self.pc + 1);

		self.pc = self.pc + 2;

		self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;
		self.instruction_address_abs = self.instruction_address_abs + self.y as u16;

		// Check if instruction is changing page and return a additional cycle if true
		if (self.instruction_address_abs & 0xFF00) != ((hi as u16) << 8) {
			return 1;
		}

		return 0;
	}

	// Hardware pointers from 6502
	fn IND(&mut self) -> u8 {
		let ptr_hi = self.bus.read(self.pc);
		let ptr_lo = self.bus.read(self.pc + 1);

		self.pc = self.pc + 2;

		let ptr_full = ((ptr_hi as u16) << 8) | ptr_lo as u16;

		if ptr_lo == 0x00FF {
			let hi = self.bus.read(ptr_full & 0xFF00 as u16);
			let lo = self.bus.read(ptr_full + 0);

			self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;
		} else {
			let hi = self.bus.read(ptr_full + 1);
			let lo = self.bus.read(ptr_full + 0);

			self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;
		}

		return 0;
	}

	fn IZX(&mut self) -> u8 {
		let pointer = self.bus.read(self.pc);

		self.pc = self.pc + 1;

		let hi = self
			.bus
			.read(((pointer as u16) + (self.x as u16)) & 0x00FF as u16);
		let lo = self
			.bus
			.read(((pointer as u16) + (self.x as u16) + 1) & 0x00FF as u16);

		self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;

		return 0;
	}

	fn IZY(&mut self) -> u8 {
		let pointer = self.bus.read(self.pc);

		self.pc = self.pc + 1;

		let hi = self.bus.read((pointer as u16) & 0x00FF as u16);
		let lo = self.bus.read(((pointer as u16) + 1) & 0x00FF as u16);

		self.instruction_address_abs = ((hi as u16) << 8) | lo as u16;
		self.instruction_address_abs = self.y as u16;

		if (self.instruction_address_abs & 0xFF00) != ((hi as u16) << 8) {
			return 1;
		}

		return 0;
	}

	// Instructions
	fn BRK(&mut self) -> u8 {
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
		return 0x00;
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
		return 0x00;
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
