const MAX_SIZE: usize = 0x1000;
const FONT_RANGE: Range<usize> = 0x0..0x200;
const ROM_RANGE: Range<usize> = 0x200..0xEA0;
const STACK_RANGE: Range<usize> = 0xEA0..0xF00;
const DISPLAY_RANGE: Range<usize> = 0xF00..MAX_SIZE;

use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{Index, IndexMut, Range};

type Address = usize;

pub struct Memory {
    mem: [u8; MAX_SIZE],
    count: usize
}

impl Memory {
    pub const ROM_RANGE: Range<usize> = ROM_RANGE;
    pub const STACK_RANGE: Range<usize> = STACK_RANGE;

    pub fn new(rom: &Vec<u8>) -> Memory {
        Memory {
            mem: Memory::init_mem(&rom),
            count: MAX_SIZE
        }
    }

    fn init_mem(rom: &Vec<u8>) -> [u8; MAX_SIZE] {
        let mut mem = [0x0; MAX_SIZE];

        for i in 0..rom.len() {
            mem[i + ROM_RANGE.start] = rom[i];
        }

        mem
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    fn index(&self, addr: Address) -> &Self::Output {
        &self.mem[addr]
    }
}

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, addr: Address) -> &mut u8 {
        &mut self.mem[addr]
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let hex_bytes = self.mem.iter()
            .map(|byte| format!("{:02x}", &byte));

        let lines = hex_bytes.enumerate()
            .fold(String::new(), |mut acc, (i, hex_byte)| {
                if i != 0 {
                    if i % 2 == 0 { acc.push_str(" "); }
                    if i % 16 == 0 { acc.push_str("\n"); }
                }

                acc.push_str(&hex_byte);
                acc
            });

        write!(f, "{}", lines)
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self)
    }
}
