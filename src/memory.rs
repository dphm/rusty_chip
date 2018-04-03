use std::cmp::PartialEq;
use std::clone::Clone;
use std::fmt::{self, Debug};
use std::ops::{Deref, Index, IndexMut, Range};

use Address;

#[derive(Clone)]
pub struct Memory<T> {
    memory: Vec<T>
}

impl<T> Memory<T> where T: Clone {
    pub fn new(size: usize, default: T) -> Memory<T> {
        let mut memory = Vec::new();
        memory.resize(size, default);

        Memory {
            memory
        }
    }

    pub fn load(&mut self, data: &[T], range: Range<Address>) {
        for i in 0..data.len() {
            if i >= range.end {
                panic!("Data length {:?} is greater than range size {:?}", data.len(), range);
            }

            self.memory[range.start + i] = data[i].clone();
        }
    }
}

impl<T> Deref for Memory<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl<T> Index<Address> for Memory<T> {
    type Output = T;

    fn index(&self, addr: Address) -> &Self::Output {
        &self.memory[addr]
    }
}

impl<T> IndexMut<Address> for Memory<T> {
    fn index_mut(&mut self, addr: Address) -> &mut T {
        &mut self.memory[addr]
    }
}

impl<T> Index<Range<Address>> for Memory<T> {
    type Output = [T];

    fn index(&self, range: Range<Address>) -> &Self::Output {
        &self.memory[range]
    }
}

impl<T> Debug for Memory<T> where T: fmt::LowerHex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hex_vals = self.memory.iter().map(|val| format!("{:02x}", &val));

        let lines = hex_vals.enumerate()
            .fold(String::new(), |mut acc, (i, hex_val)| {
                if i != 0 && i % 2 == 0 { acc.push_str(" "); }
                if i % 16 == 0 { acc.push_str("\n"); }

                acc.push_str(&hex_val);
                acc
            });

        write!(f, "{}", lines)
    }
}

impl<T> PartialEq for Memory<T> where T: PartialEq {
    fn eq(&self, other: &Memory<T>) -> bool {
        self.memory == other.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Byte;

    #[test]
    fn load_data_with_len_equal_range() {
        let data: Vec<Byte> = vec![0xA, 0xB, 0xC, 0xD, 0xE];
        let range: Range<Address> = 0x0..0x5;
        let mut memory = Memory::new(0x1000, 0x0);

        memory.load(&data, range);
        assert_eq!(0xA, memory[0x0]);
        assert_eq!(0xB, memory[0x1]);
        assert_eq!(0xC, memory[0x2]);
        assert_eq!(0xD, memory[0x3]);
        assert_eq!(0xE, memory[0x4]);
    }

    #[test]
    fn load_data_with_len_less_than_range() {
        let data: Vec<Byte> = vec![0xA, 0xB, 0xC];
        let range: Range<Address> = 0x0..0x5;
        let mut memory = Memory::new(0x1000, 0x0);

        memory.load(&data, range);
        assert_eq!(0xA, memory[0x0]);
        assert_eq!(0xB, memory[0x1]);
        assert_eq!(0xC, memory[0x2]);
    }

    #[test]
    #[should_panic(expected = "Data length")]
    fn load_data_with_len_greater_than_range_panics() {
        let data: Vec<Byte> = vec![0xA, 0xB, 0xC, 0xD, 0xE, 0xF];
        let range: Range<Address> = 0x0..0x5;
        let mut memory = Memory::new(0x1000, 0x0);

        memory.load(&data, range);
    }
}
