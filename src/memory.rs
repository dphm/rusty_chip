const PROGRAM_SIZE: usize = 0x1000 - 0x200;
const STACK_SIZE: usize = 16;

use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{Index, IndexMut};

type Address = usize;

pub struct Memory {
    program: [u8; PROGRAM_SIZE],
    stack: Vec<Address>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            program: [0x0; PROGRAM_SIZE],
            stack: Vec::with_capacity(STACK_SIZE)
        }
    }

    pub fn stack_push(&mut self, addr: Address) {
        self.stack.push(addr)
    }

    pub fn stack_pop(&mut self) -> Option<Address> {
        self.stack.pop()
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    fn index(&self, addr: Address) -> &u8 {
        &self.program[addr]
    }
}

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, addr: Address) -> &mut u8 {
        &mut self.program[addr]
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let hex_bytes = self.program.iter()
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
        let hex_stack: Vec<String> = self.stack.iter()
            .map(|addr| format!("{:04x}", &addr))
            .collect();

        write!(f, "STACK: [{}]\n\n{}", hex_stack.join(", "), self)
    }
}
