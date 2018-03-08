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
use pointer::Pointer;
use opcode::Opcode;
use output::font;

type Address = usize;
type Byte = u8;

#[derive(Debug)]
pub struct Cpu<'a> {
    pub exit: bool,
    pc: Pointer,
    sp: Pointer,
    i: Pointer,
    dt: Timer,
    st: Timer,
    v: [Byte; NUM_REGISTERS],
    memory: &'a mut Memory
}

impl<'a> Cpu<'a> {
    pub fn new(memory: &'a mut Memory, rom: &Vec<Byte>) -> Cpu<'a> {
        Cpu {
            exit: false,
            pc: Pointer::new(ROM_RANGE),
            sp: Pointer::new(STACK_RANGE),
            i: Pointer::new(ROM_RANGE),
            dt: Timer::new(60),
            st: Timer::new(60),
            v: [0x0; NUM_REGISTERS],
            memory: memory
                .load(&font::FONT_SET, &FONT_RANGE)
                .load(&rom, &ROM_RANGE)
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute(&opcode);
        self.pc.move_forward();

        self.dt.tick();
        self.st.tick();
    }

    fn fetch_opcode(&mut self) -> Opcode {
        let current = self.pc.current;
        let bytes = (self.memory[current], self.memory[current + 1]);
        Opcode::from_bytes(bytes)
    }

    fn execute(&mut self, opcode: &Opcode) {
        match opcode.first_hex_digit() {
            0x0 => {
                match opcode.kk() {
                    0xE0 => self.clear_display(),
                    0xEE => self.return_from_subroutine(),
                    _ => self.unknown_opcode(&opcode)
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
                    _ => self.unknown_opcode(&opcode)
                }
            },
            0x9 => {
                let vx = self.v[opcode.x()];
                let vy = self.v[opcode.y()];
                self.skip_if(vx != vy);
            },
            0xA => {
                self.i.set(opcode.nnn());
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
                    _ => self.unknown_opcode(&opcode)
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
                        let addr = self.i.current.wrapping_add(vx as Address);
                        self.i.set(addr);

                    },
                    0x29 => {
                        let vx = self.v[x];
                        let addr = font::sprite_addr(vx);
                        self.i.set(addr);
                    },
                    0x33 => self.store_bcd(x),
                    0x55 => self.store_registers_through(x),
                    0x65 => self.read_registers_through(x),
                    _ => self.unknown_opcode(&opcode)
                }
            },
            _ => self.unknown_opcode(&opcode)
        }
    }

    fn unknown_opcode(&self, opcode: &Opcode) {
        panic!("Unknown opcode {:?}", opcode)
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
        let current = self.sp.current;
        let addr = (self.memory[current] as Address) << 8 | (self.memory[current + 1] as Address);
        self.sp.move_backward();
        addr
    }

    fn stack_push(&mut self, addr: Address) {
        self.sp.move_forward();
        let current = self.sp.current;
        self.memory[current] = ((addr & 0xFF00) >> 8) as Byte;
        self.memory[current + 1] = (addr & 0x00FF) as Byte;
    }

    fn return_from_subroutine(&mut self) {
        let addr = self.stack_pop();
        self.pc.set(addr);
    }

    fn jump(&mut self, addr: Address) {
        self.pc.set(addr);
    }

    fn call_subroutine(&mut self, addr: Address) {
        let current = self.pc.current;
        self.stack_push(current);
        self.pc.set(addr);
    }

    fn skip_if(&mut self, p: bool) {
        if p { self.pc.move_forward(); }
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
        let i = self.i.current;
        self.memory[i] = vx / 100;
        self.memory[i + 1] = vx % 100 / 10;
        self.memory[i + 2] = vx % 10;
    }

    fn store_registers_through(&mut self, x: Address) {
        for i in 0..x + 1 {
            self.memory[self.i.current + i] = self.v[i];
        }
    }

    fn read_registers_through(&mut self, x: Address) {
        for i in 0..x + 1 {
            self.v[i] = self.memory[self.i.current + i];
        }
    }

    fn draw_sprite(&mut self, x: Address, y: Address, n: usize) {
        // Draw n-byte sprite with memory starting at I at (x, y)
        // Set flag if collision
    }
}
