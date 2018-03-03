const NUM_REGISTERS: usize = 16;

extern crate rand;

use self::rand::Rng;
use memory::Memory;
use timer::Timer;
use io::lcd::Lcd;
use io::font;

type Address = usize;
type Opcode = u16;
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
    memory: &'a mut Memory,
    lcd: &'a mut Lcd
}

impl<'a> Cpu<'a> {
    pub fn new(memory: &'a mut Memory, lcd: &'a mut Lcd) -> Cpu<'a> {
        Cpu {
            exit: false,
            pc: Memory::ROM_RANGE.start,
            sp: Memory::STACK_RANGE.start,
            i: 0x0,
            dt: Timer::new(60),
            st: Timer::new(60),
            v: [0x0; NUM_REGISTERS],
            memory,
            lcd
        }
    }

    pub fn step(&mut self) {
        let opcode: Opcode = self.fetch();
        self.execute(opcode);

        self.dt.tick();
        self.st.tick();

        if self.pc + 1 >= Memory::ROM_RANGE.end {
            self.exit = true;
            println!("{:?}", self);
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
            0x0000 => {
                match opcode & 0x00FF {
                    0xE0 => self.clear_display(),
                    0xEE => self.return_from_subroutine(),
                    _ => ()
                }
            },
            0x1000 => {
                let nnn = (opcode & 0x0FFF) as Address;
                self.jump(nnn);
            },
            0x2000 => {
                let nnn = (opcode & 0x0FFF) as Address;
                self.call_subroutine(nnn);
            },
            0x3000 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                let vx = self.v[x];
                self.skip_if_equal(vx, byte);
            },
            0x4000 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                let vx = self.v[x];
                self.skip_if_not_equal(vx, byte);
            },
            0x5000 => {
                let x = (opcode & 0x0F00) as Address;
                let y = (opcode & 0x00F0) as Address;
                let vx = self.v[x];
                let vy = self.v[y];
                self.skip_if_equal(vx, vy);
            },
            0x6000 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                self.load(x, byte);
            },
            0x7000 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                self.add(x, byte);
            },
            0x8000 => {
                let x = (opcode & 0x0F00) as Address;
                let y = (opcode & 0x00F0) as Address;
                let f = (opcode & 0x000F) as Byte;
                match f {
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
            0x9000 => {
                let x = (opcode & 0x0F00) as Address;
                let y = (opcode & 0x00F0) as Address;
                let vx = self.v[x];
                let vy = self.v[y];
                self.skip_if_not_equal(vx, vy);
            },
            0xA000 => {
                let nnn = (opcode & 0x0FFF) as Address;
                self.load_i(nnn);
            },
            0xB000 => {
                let nnn = (opcode & 0x0FFF) as Address;
                let v0 = self.v[0];
                self.jump(nnn + (v0 as Address));
            },
            0xC000 => {
                let x = (opcode & 0x0F00) as Address;
                let byte = (opcode & 0x00FF) as Byte;
                self.random_and(x, byte);
            },
            0xD000 => {
                let x = (opcode & 0x0F00) as Address;
                let y = (opcode & 0x00F0) as Address;
                let n = (opcode & 0x000F) as usize;
                self.draw_sprite(x, y, n);
            },
            0xE000 => {
                let x = (opcode & 0x0F00) as Address;
                match opcode & 0x00FF {
                    0x9E => (), // skip if key[v[x]] down
                    0xA1 => (), // skip if key[v[x]] up
                    _ => ()
                }
            },
            0xF000 => {
                let x = (opcode & 0x0F00) as Address;
                match opcode & 0x00FF {
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
                        let addr = font::sprite_addr(vx as Address);
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

    fn set_flag(&mut self, val: Byte) {
        self.v[0xF] = val;
    }

    fn clear_display(&mut self) {
        self.memory.clear(&Memory::DISPLAY_RANGE)
    }

    fn return_from_subroutine(&mut self) {
        let addr: Address = self.memory.stack_pop(&self.sp);
        self.sp -= 2;
        self.pc = addr;
    }

    fn jump(&mut self, addr: Address) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: Address) {
        self.memory.stack_push(&self.sp, &self.pc);
        self.sp += 2;
        self.pc = addr;
    }

    fn skip_if_equal(&mut self, a: Byte, b: Byte) {
        if a == b { self.pc += 2; }
    }

    fn skip_if_not_equal(&mut self, a: Byte, b: Byte) {
        if a != b { self.pc += 2; }
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
        self.v[x]= vy.wrapping_sub(vx);

        if vx < vy {
            self.set_flag(0b1);
        } else {
            self.set_flag(0b0);
        }
    }

    fn shift_right(&mut self, x: Address) {
        let mut bit_string = format!("{:b}", x);
        let last = (bit_string.len() - 1) as usize;
        match bit_string.remove(last) {
            '1' => self.set_flag(0b1),
            '0' => self.set_flag(0b0),
            _ => panic!("Failed to set flag")
        }

        self.v[x] = self.v[x] >> 1;
    }

    fn shift_left(&mut self, x: Address) {
        let mut bit_string = format!("{:b}", x);
        match bit_string.remove(0) {
            '1' => self.set_flag(0b1),
            '0' => self.set_flag(0b0),
            _ => panic!("Failed to set flag")
        }

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
        let point = (self.v[x], self.v[y]);
        for i in 0..n {
            let collision: bool = self.lcd.draw((self.v[x], self.v[y]), self.memory[self.i + i]);
            if collision {
                self.set_flag(0b1);
            } else {
                self.set_flag(0b0);
            }
        }
    }
}
