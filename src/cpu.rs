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
        self.execute(opcode);

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

    fn execute(&mut self, opcode: Opcode) {
        match opcode & 0xF000 {
            0x0 => {
                match opcode & 0x00FF {
                    0xE0 => self.clear_display(),
                    0xEE => self.return_from_subroutine(),
                    _ => ()
                }
            },
            0x1 => {
                let nnn = (opcode & 0x0FFF) as Address;
                self.jump(nnn);
            },
            0x2 => {
                let nnn = (opcode & 0x0FFF) as Address;
                self.call_subroutine(nnn);
            },
            0x3 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                let vx = self.v[x];
                self.skip_if_equal(vx, byte);
            },
            0x4 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                let vx = self.v[x];
                self.skip_if_not_equal(vx, byte);
            },
            0x5 => {
                let x = (opcode & 0x0F00) as Address;
                let y = (opcode & 0x00F0) as Address;
                let vx = self.v[x];
                let vy = self.v[y];
                self.skip_if_equal(vx, vy);
            },
            _ => ()
        }
    }

    fn current_val(&self) -> Byte {
        self.memory[self.pc]
    }

    fn advance_pc(&mut self) {
        self.pc += 1;
    }

    fn clear_display(&mut self) {
        self.memory.clear(Memory::DISPLAY_RANGE)
    }

    fn return_from_subroutine(&mut self) {
        let addr: Address = self.memory.stack_pop(self.sp);
        self.sp -= 2;
        self.pc = addr;
    }

    fn jump(&mut self, addr: Address) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: Address) {
        self.memory.stack_push(self.sp, self.pc);
        self.sp += 2;
        self.pc = addr;
    }

    fn skip_if_equal(&mut self, a: Byte, b: Byte) {
        if a == b { self.pc += 2; }
    }

    fn skip_if_not_equal(&mut self, a: Byte, b: Byte) {
        if a != b { self.pc += 2; }
    }
}
