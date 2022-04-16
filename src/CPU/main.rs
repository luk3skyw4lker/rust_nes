#![allow(non_snake_case)]
pub mod instructions;

pub struct CPU {
	pc: u16,    // Program Counter
	a: u8,      // Register A
	x: u8,      // Register X
	y: u8,      // Register Y
	sp: u8,     // Stack Pointer
	status: u8, // Status Register (https://www.nesdev.org/wiki/Status_flags)
	cycles: u8, // Cycles Counter
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
		}
	}

	// Internal Functions
	pub fn reset(&mut self) -> () {
		const PC_INITIAL_ADDRESS: u16 = 0xFFCC;

		self.pc = PC_INITIAL_ADDRESS;

		self.a = 0;
		self.x = 0;
		self.y = 0;
		self.sp = 0xFD;
		self.status = 0x00 | (1 << 5);

		self.cycles = 8;
	}
	pub fn irq(&mut self) -> () {}
	pub fn nmi(&mut self) -> () {}
	pub fn step(&mut self) -> () {
		match self.cycles {
			0 => println!("execute instruction"),
			_ => println!("stall"),
		}
	}

	// Addresssing Modes
	fn IMP() -> () {}
	fn IMM() -> () {}
	fn ZP0() -> () {}
	fn ZPX() -> () {}
	fn ZPY() -> () {}
	fn REL() -> () {}
	fn ABS() -> () {}
	fn ABX() -> () {}
	fn ABY() -> () {}
	fn IND() -> () {}
	fn IZX() -> () {}
	fn IZY() -> () {}

	// Instructions
	fn BRK(&self) -> () {}
	fn ORA(&self) -> () {}
	fn ASL(&self) -> () {}
	fn PHP(&self) -> () {}
	fn BPL(&self) -> () {}
	fn CLC(&self) -> () {}
	fn JSR(&self) -> () {}
	fn AND(&self) -> () {}
	fn BIT(&self) -> () {}
	fn ROL(&self) -> () {}
	fn PLP(&self) -> () {}
	fn BMI(&self) -> () {}
	fn SEC(&self) -> () {}
	fn RTI(&self) -> () {}
	fn EOR(&self) -> () {}
	fn LSR(&self) -> () {}
	fn PHA(&self) -> () {}
	fn PLA(&self) -> () {}
	fn JMP(&self) -> () {}
	fn BVC(&self) -> () {}
	fn CLI(&self) -> () {}
	fn RTS(&self) -> () {}
	fn ADC(&self) -> () {}
	fn ROR(&self) -> () {}
	fn BVS(&self) -> () {}
	fn SEI(&self) -> () {}
	fn STA(&self) -> () {}
	fn STY(&self) -> () {}
	fn STX(&self) -> () {}
	fn DEY(&self) -> () {}
	fn TXA(&self) -> () {}
	fn BCC(&self) -> () {}
	fn TYA(&self) -> () {}
	fn TXS(&self) -> () {}
	fn LDY(&self) -> () {}
	fn LDA(&self) -> () {}
	fn LDX(&self) -> () {}
	fn TAY(&self) -> () {}
	fn TAX(&self) -> () {}
	fn BCS(&self) -> () {}
	fn CLV(&self) -> () {}
	fn TSX(&self) -> () {}
	fn CPY(&self) -> () {}
	fn CMP(&self) -> () {}
	fn DEC(&self) -> () {}
	fn DEX(&self) -> () {}
	fn INY(&self) -> () {}
	fn BNE(&self) -> () {}
	fn CLD(&self) -> () {}
	fn CPX(&self) -> () {}
	fn SBC(&self) -> () {}
	fn INC(&self) -> () {}
	fn INX(&self) -> () {}
	fn BEQ(&self) -> () {}
	fn SED(&self) -> () {}
	// illegal opcode function, all illegal opcodes will be mapped to this function
	fn NOP(&self) -> () {}
}
