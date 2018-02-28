const NUM_REGISTERS: usize = 16;

use memory::Memory;
use timer::Timer;

type Address = usize;
type Opcode = u16;
type Byte = u8;

pub struct Cpu<'a> {
    pub exit: bool,
    pc: Address,
    sp: Address,
    i: Address,
    dt: Timer,
    st: Timer,
    v: [Byte; NUM_REGISTERS],
    memory: &'a mut Memory
}

impl<'a> Cpu<'a> {
    pub fn new(memory: &'a mut Memory) -> Cpu<'a> {
        Cpu {
            exit: false,
            pc: Memory::ROM_RANGE.start,
            sp: Memory::STACK_RANGE.start,
            i: 0x0,
            dt: Timer::new(60),
            st: Timer::new(60),
            v: [0x0; NUM_REGISTERS],
            memory
        }
    }

    pub fn step(&mut self) {
        let opcode: Opcode = self.fetch();
        let operation = self.decode(opcode);
        operation();

        if self.pc + 1 >= Memory::ROM_RANGE.end {
            self.exit = true;
            println!("{}", self.memory);
            return;
        }
    }

    fn fetch(&mut self) -> Opcode {
        let op_a = self.current_val() as Opcode;
        self.advance_pc();
        let op_b = self.current_val() as Opcode;

        op_a << 8 | op_b
    }

    fn decode(&self, opcode: Opcode) -> fn() {
        match opcode {
            _ => no_op
        }
    }

    fn current_val(&self) -> Byte {
        self.memory[self.pc]
    }

    fn advance_pc(&mut self) {
        self.pc += 1;
    }
}

fn no_op() {}
