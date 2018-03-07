use std::fmt::{self, Debug};
use std::ops::{Index, IndexMut, Range};

type Address = usize;
type Byte = u8;

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

    pub fn load(&mut self, data: &[Byte], range: &Range<Address>) -> &mut Memory {
        for i in 0..data.len() {
            if i >= range.end {
                panic!("Data length {:?} is greater than range size {:?}", data.len(), range);
            }

            self.mem[range.start + i] = data[i];
        }

        self
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

impl Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hex_bytes = self.mem.iter()
            .map(|byte| format!("{:02x}", &byte));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_data_with_len_equal_range() {
        let data: Vec<Byte> = vec![0xA, 0xB, 0xC, 0xD, 0xE];
        let range: Range<Address> = 0x0..0x5;
        let mut memory = Memory::new();

        memory.load(&data, &range);
        assert_eq!(0xA, memory.mem[0x0]);
        assert_eq!(0xB, memory.mem[0x1]);
        assert_eq!(0xC, memory.mem[0x2]);
        assert_eq!(0xD, memory.mem[0x3]);
        assert_eq!(0xE, memory.mem[0x4]);
    }

    #[test]
    fn load_data_with_len_less_than_range() {
        let data: Vec<Byte> = vec![0xA, 0xB, 0xC];
        let range: Range<Address> = 0x0..0x5;
        let mut memory = Memory::new();

        memory.load(&data, &range);
        assert_eq!(0xA, memory.mem[0x0]);
        assert_eq!(0xB, memory.mem[0x1]);
        assert_eq!(0xC, memory.mem[0x2]);
    }

    #[test]
    #[should_panic(expected = "Data length")]
    fn load_data_with_len_greater_than_range_panics() {
        let data: Vec<Byte> = vec![0xA, 0xB, 0xC, 0xD, 0xE, 0xF];
        let range: Range<Address> = 0x0..0x5;
        let mut memory = Memory::new();

        memory.load(&data, &range);
    }
}
