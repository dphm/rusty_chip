use std::fmt::{self, Debug};
use std::ops::{Index, IndexMut, Range};

use {Address, Byte};

pub struct Memory {
    mem: [Byte; Memory::MAX_SIZE]
}

impl Memory {
    pub const MAX_SIZE: usize = 0x1000;

    pub fn new() -> Memory {
        Memory {
            mem: [0x0; Memory::MAX_SIZE]
        }
    }

    pub fn load(&mut self, data: &[Byte], range: Range<Address>) {
        for i in 0..data.len() {
            if i >= range.end {
                panic!("Data length {:?} is greater than range size {:?}", data.len(), range);
            }

            self.mem[range.start + i] = data[i];
        }
    }
}

impl Index<Address> for Memory {
    type Output = Byte;

    fn index(&self, addr: Address) -> &Self::Output {
        &self.mem[addr]
    }
}

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, addr: Address) -> &mut Byte {
        &mut self.mem[addr]
    }
}

impl Index<Range<Address>> for Memory {
    type Output = [Byte];

    fn index(&self, range: Range<Address>) -> &Self::Output {
        &self.mem[range]
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hex_bytes = self.mem.iter().map(|byte| format!("{:02x}", &byte));

        let lines = hex_bytes.enumerate()
            .fold(String::new(), |mut acc, (i, hex_byte)| {
                if i != 0 && i % 2 == 0 { acc.push_str(" "); }
                if i % 16 == 0 { acc.push_str("\n"); }

                acc.push_str(&hex_byte);
                acc
            });

        write!(f, "{}", lines)
    }
}
