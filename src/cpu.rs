const REG_SIZE: usize = 16;

use memory::Memory;

type Address = usize;
type Opcode = u16;

pub struct Cpu<'a> {
    pc: Address,
    sp: Address,
    i: Address,
    v: [u8; REG_SIZE],
    memory: &'a mut Memory
}

impl<'a> Cpu<'a> {
    pub fn new(memory: &'a mut Memory) -> Cpu<'a> {
        Cpu {
            pc: 0x0,
            sp: 0x0,
            i: 0x0,
            v: [0x0; REG_SIZE],
            memory
        }
    }

    pub fn step(&mut self) {
        let opcode: Opcode = self.fetch();
        let operation = self.decode(opcode);
        operation();
    }

    fn fetch(&mut self) -> Opcode {
        let op_a = self.current_val() as u16;
        self.advance_pc();
        let op_b = self.current_val() as u16;

        op_a << 8 | op_b
    }

    fn decode(&self, opcode: Opcode) -> fn() {
        match opcode {
            _ => no_op
        }
    }

    fn current_val(&self) -> u8 {
        self.memory[self.pc]
    }

    fn advance_pc(&mut self) {
        self.pc = self.pc + 1
    }
}

fn no_op() {}
