const MAX_SIZE: usize = 0x1000;
const FONT_RANGE: Range<Address> = 0x0..0x200;
const ROM_RANGE: Range<Address> = 0x200..0xEA0;
const STACK_RANGE: Range<Address> = 0xEA0..0xF00;
const DISPLAY_RANGE: Range<Address> = 0xF00..MAX_SIZE;

use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{Index, IndexMut, Range};
use io::font;

type Address = usize;
type Byte = u8;

pub struct Memory {
    mem: [Byte; MAX_SIZE]
}

impl Memory {
    pub const ROM_RANGE: Range<Address> = ROM_RANGE;
    pub const STACK_RANGE: Range<Address> = STACK_RANGE;
    pub const DISPLAY_RANGE: Range<Address> = 0xF00..MAX_SIZE;

    pub fn new(rom: &Vec<Byte>) -> Memory {
        Memory {
            mem: Memory::init_mem(&rom)
        }
    }

    pub fn clear(&mut self, range: &Range<Address>) {
        for i in range.start..range.end {
            self.mem[i] = 0x0;
        }
    }

    pub fn stack_pop(&mut self, sp: &Address) -> Address {
        (self.mem[*sp] as Address) << 8 | (self.mem[*sp + 1] as Address)
    }

    pub fn stack_push(&mut self, sp: &Address, addr: &Address) {
        self.mem[*sp] = ((*addr & 0xFF00) >> 8) as Byte;
        self.mem[*sp + 1] = (*addr & 0x00FF) as Byte;
    }

    fn init_mem(rom: &Vec<Byte>) -> [Byte; MAX_SIZE] {
        let mut mem = [0x0; MAX_SIZE];

        Memory::load_font(&mut mem);
        Memory::load_rom(&mut mem, &rom);

        mem
    }

    fn load_font(mem: &mut [Byte]) {
        for i in 0..font::FONT_SET.len() {
            mem[FONT_RANGE.start + i] = font::FONT_SET[i];
        }
    }

    fn load_rom(mem: &mut [Byte], rom: &Vec<Byte>) {
        for i in 0..rom.len() {
            mem[i + ROM_RANGE.start] = rom[i];
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

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter) -> Result {
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

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn new_loads_font() {
        let r = Vec::new();
        let m = Memory::new(&r);
        let font_set: [Byte; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for i in 0..font_set.len() {
            assert_eq!(font_set[i], m[FONT_RANGE.start + i]);
        }
    }

    #[test]
    fn new_loads_rom() {
        let mut r = Vec::new();
        let bytes: [Byte; 4] = [0xAB, 0xCD, 0xEF, 0x60];
        for byte in bytes.iter() {
            r.push(*byte);
        }

        let m = Memory::new(&r);
        for i in 0..bytes.len() {
            assert_eq!(bytes[0 + i], m[ROM_RANGE.start + i]);
        }
    }

    #[test]
    fn clear_resets_bytes_in_range() {
        let clear_range: Range<Address> = (FONT_RANGE.start + 10)..(FONT_RANGE.start + 20);
        let r = Vec::new();
        let mut m = Memory::new(&r);
        let expected: [Byte; 30] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0x00, 0x00, 0x00, 0x00, 0x00, // 2
            0x00, 0x00, 0x00, 0x00, 0x00, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        ];

        m.clear(&clear_range);
        for i in 0..expected.len() {
            assert_eq!(expected[i], m[FONT_RANGE.start + i]);
        }
    }

    #[test]
    fn stack_push() {
        let r = Vec::new();
        let mut m = Memory::new(&r);
        let sp: Address = STACK_RANGE.start;
        let addr: Address = 0xABCD;

        m.stack_push(&sp, &addr);
        assert_eq!(0xAB, m[sp]);
        assert_eq!(0xCD, m[sp + 1]);
    }

    #[test]
    fn stack_pop() {
        let r = Vec::new();
        let mut m = Memory::new(&r);
        let sp: Address = STACK_RANGE.start;
        m[sp] = 0xAB;
        m[sp + 1] = 0xCD;

        let addr: Address = m.stack_pop(&sp);
        assert_eq!(0xABCD, addr);
    }
}
