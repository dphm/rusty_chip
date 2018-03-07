const NUM_REGISTERS: usize = 16;

const FONT_RANGE: Range<Address> = 0x0..0x200;
const ROM_RANGE: Range<Address> = 0x200..0xEA0;
const STACK_RANGE: Range<Address> = 0xEA0..0xF00;
const DISPLAY_RANGE: Range<Address> = 0xF00..Memory::MAX_SIZE;

extern crate rand;

use self::rand::Rng;
use std::ops::Range;

use memory::Memory;
use timer::Timer;
use opcode::Opcode;
use output::font;

type Address = usize;
type Byte = u8;

#[derive(Debug)]
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
    pub fn new(memory: &'a mut Memory, rom: &Vec<Byte>) -> Cpu<'a> {
        Cpu {
            exit: false,
            pc: ROM_RANGE.start,
            sp: STACK_RANGE.start - 2,
            i: 0x0,
            dt: Timer::new(60),
            st: Timer::new(60),
            v: [0x0; NUM_REGISTERS],
            memory: memory
                .load(&font::FONT_SET, &FONT_RANGE)
                .load(&rom, &ROM_RANGE)
        }
    }

    pub fn step(&mut self) {
        let opcode: Opcode = self.fetch();
        self.execute(&opcode);

        self.dt.tick();
        self.st.tick();

        if self.pc + 1 >= ROM_RANGE.end {
            self.exit = true;
            println!("{:?}", self);
            return;
        }
    }

    fn fetch(&mut self) -> Opcode {
        let a = self.current_val();
        self.advance_pc();
        let b = self.current_val();

        Opcode::from_bytes((a, b))
    }

    fn execute(&mut self, opcode: &Opcode) {
        match opcode.first_hex_digit() {
            0x0 => {
                match opcode.kk() {
                    0xE0 => self.clear_display(),
                    0xEE => self.return_from_subroutine(),
                    _ => ()
                }
            },
            0x1 => self.jump(opcode.nnn()),
            0x2 => self.call_subroutine(opcode.nnn()),
            0x3 => {
                let vx = self.v[opcode.x()];
                self.skip_if(vx == opcode.kk());
            },
            0x4 => {
                let vx = self.v[opcode.x()];
                self.skip_if(vx != opcode.kk());
            },
            0x5 => {
                let vx = self.v[opcode.x()];
                let vy = self.v[opcode.y()];
                self.skip_if(vx == vy);
            },
            0x6 => {
                self.load(opcode.x(), opcode.kk());
            },
            0x7 => {
                self.add(opcode.x(), opcode.kk());
            },
            0x8 => {
                let x = opcode.x();
                let y = opcode.y();
                match opcode.k() {
                    0x1 => self.or(x, y),
                    0x2 => self.and(x, y),
                    0x3 => self.xor(x, y),
                    0x4 => self.add_with_carry(x, y),
                    0x5 => self.subtract_without_borrow(x, y),
                    0x6 => self.shift_right(x),
                    0x7 => self.subtract_neg_without_borrow(x, y),
                    0xE => self.shift_left(x),
                    _ => ()
                }
            },
            0x9 => {
                let vx = self.v[opcode.x()];
                let vy = self.v[opcode.y()];
                self.skip_if(vx != vy);
            },
            0xA => {
                self.load_i(opcode.nnn());
            },
            0xB => {
                let v0 = self.v[0];
                self.jump(opcode.nnn() + (v0 as Address));
            },
            0xC => {
                self.random_and(opcode.x(), opcode.kk());
            },
            0xD => {
                self.draw_sprite(opcode.x(), opcode.y(), opcode.k() as usize);
            },
            0xE => {
                match opcode.kk() {
                    0x9E => (), // skip if key[v[x]] down
                    0xA1 => (), // skip if key[v[x]] up
                    _ => ()
                }
            },
            0xF => {
                let x = opcode.x();
                match opcode.kk() {
                    0x07 => {
                        let dt = self.dt.current;
                        self.load(x, dt);
                    },
                    0x0A => (), // wait for key press, store value in Vx.
                    0x15 => {
                        let vx = self.v[x];
                        self.dt.set(vx);
                    },
                    0x18 => {
                        let vx = self.v[x];
                        self.st.set(vx);
                    },
                    0x1E => {
                        let vx = self.v[x];
                        let addr = self.i.wrapping_add(vx as Address);
                        self.load_i(addr);
                    },
                    0x29 => {
                        let vx = self.v[x];
                        let addr = font::sprite_addr(vx);
                        self.load_i(addr);
                    },
                    0x33 => self.store_bcd(x),
                    0x55 => self.store_registers_through(x),
                    0x65 => self.read_registers_through(x),
                    _ => ()
                }
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

    fn advance_sp(&mut self) {
        self.sp += 2;
    }

    fn retract_sp(&mut self) {
        self.sp -= 2;
    }

    fn set_flag(&mut self, val: Byte) {
        self.v[0xF] = val;
    }

    fn clear_display(&mut self) {
        for i in DISPLAY_RANGE {
            self.memory[i] = 0x0;
        }
    }

    fn stack_pop(&mut self) -> Address {
        (self.memory[self.sp] as Address) << 8 | (self.memory[self.sp + 1] as Address)
    }

    fn stack_push(&mut self, addr: Address) {
        if self.sp + 1 >= STACK_RANGE.end { panic!("Stack overflow"); }
        self.memory[self.sp] = ((addr & 0xFF00) >> 8) as Byte;
        self.memory[self.sp + 1] = (addr & 0x00FF) as Byte;
    }

    fn return_from_subroutine(&mut self) {
        if self.sp < STACK_RANGE.start { return; }
        self.pc = self.stack_pop();
        self.retract_sp();
    }

    fn jump(&mut self, addr: Address) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: Address) {
        let current = self.pc;
        self.advance_sp();
        self.stack_push(current);
        self.pc = addr;
    }

    fn skip_next_instruction(&mut self) {
        self.pc += 2;
    }

    fn skip_if(&mut self, p: bool) {
        if p { self.skip_next_instruction(); }
    }

    fn load_i(&mut self, addr: Address) {
        self.i = addr;
    }

    fn load(&mut self, x: Address, b: Byte) {
        self.v[x] = b;
    }

    fn add(&mut self, x: Address, b: Byte) {
        self.v[x] = self.v[x].wrapping_add(b);
    }

    fn or(&mut self, x: Address, y: Address) {
        self.v[x] = self.v[x] | self.v[y];
    }

    fn and(&mut self, x: Address, y: Address) {
        self.v[x] = self.v[x] & self.v[y];
    }

    fn xor(&mut self, x: Address, y: Address) {
        self.v[x] = self.v[x] ^ self.v[y];
    }

    fn add_with_carry(&mut self, x: Address, y: Address) {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vx.wrapping_add(vy);

        if (vx as u16) + (vy as u16) > 0xFF {
            self.set_flag(0b1);
        } else {
            self.set_flag(0b0);
        }
    }

    fn subtract_without_borrow(&mut self, x: Address, y: Address) {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vx.wrapping_sub(vy);

        if vx >= vy {
            self.set_flag(0b1);
        } else {
            self.set_flag(0b0);
        }
    }

    fn subtract_neg_without_borrow(&mut self, x: Address, y: Address) {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vy.wrapping_sub(vx);

        if vx <= vy {
            self.set_flag(0b1);
        } else {
            self.set_flag(0b0);
        }
    }

    fn shift_right(&mut self, x: Address) {
        let least_sig = self.v[x] % 2;
        self.set_flag(least_sig);
        self.v[x] = self.v[x] >> 1;
    }

    fn shift_left(&mut self, x: Address) {
        let most_sig = self.v[x] >> 7;
        self.set_flag(most_sig);
        self.v[x] = self.v[x] << 1;
    }

    fn random_and(&mut self, x: Address, byte: Byte) {
        let random_byte: Byte = rand::thread_rng().gen_range(0x0, 0xFF);
        self.v[x] = byte & random_byte;
    }

    fn store_bcd(&mut self, x: Address) {
        let vx = self.v[x];
        self.memory[self.i] = vx / 100;
        self.memory[self.i + 1] = vx % 100 / 10;
        self.memory[self.i + 2] = vx % 10;
    }

    fn store_registers_through(&mut self, x: Address) {
        for i in 0..x + 1 {
            self.memory[self.i + i] = self.v[i];
        }
    }

    fn read_registers_through(&mut self, x: Address) {
        for i in 0..x + 1 {
            self.v[i] = self.memory[self.i + i];
        }
    }

    fn draw_sprite(&mut self, x: Address, y: Address, n: usize) {
        // Draw n-byte sprite with memory starting at I at (x, y)
        // Set flag if collision
    }
}
