use std::ops::{Index, IndexMut};

const PROGRAM_SIZE: usize = 0x1000 - 0x200;
const STACK_SIZE: usize = 16;

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
