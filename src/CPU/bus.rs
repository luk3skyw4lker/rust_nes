const MEMORY_SIZE: usize = 64 * 1024;

pub struct Bus {
	ram: [u8; MEMORY_SIZE],
}

impl Bus {
	pub fn new() -> Self {
		Self {
			ram: [0x00; MEMORY_SIZE],
		}
	}

	pub fn write(&mut self, address: u16, data: u8) -> () {
		// if let 0..=65535 = address {
		self.ram[address as usize] = data;
		// }
	}

	pub fn read(&mut self, address: u16) -> u8 {
		// if let 0..=65535 = address {
		return self.ram[address as usize];
		// }

		// return 0x00;
	}
}
