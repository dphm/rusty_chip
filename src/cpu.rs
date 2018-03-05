const NUM_REGISTERS: usize = 16;

extern crate rand;

use self::rand::Rng;
use memory::Memory;
use timer::Timer;
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
    memory: &'a mut Memory
}

impl<'a> Cpu<'a> {
    pub fn new(memory: &'a mut Memory) -> Cpu<'a> {
        Cpu {
            exit: false,
            pc: Memory::ROM_RANGE.start,
            sp: Memory::STACK_RANGE.start - 2,
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

        (op_a << 8) | op_b
    }

    fn execute(&mut self, opcode: Opcode) {
        let nnn = (opcode & 0x0FFF) as Address;
        let x = ((opcode & 0x0F00) >> 8) as Address;
        let y = ((opcode & 0x00F0) >> 4) as Address;
        let kk = (opcode & 0x00FF) as Byte;
        let k = (opcode & 0x000F) as Byte;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0xE0 => self.clear_display(),
                    0xEE => self.return_from_subroutine(),
                    _ => ()
                }
            },
            0x1000 => {
                self.jump(nnn);
            },
            0x2000 => {
                self.call_subroutine(nnn);
            },
            0x3000 => {
                let vx = self.v[x];
                self.skip_if_equal(vx, kk);
            },
            0x4000 => {
                let vx = self.v[x];
                self.skip_if_not_equal(vx, kk);
            },
            0x5000 => {
                let vx = self.v[x];
                let vy = self.v[y];
                self.skip_if_equal(vx, vy);
            },
            0x6000 => {
                self.load(x, kk);
            },
            0x7000 => {
                self.add(x, kk);
            },
            0x8000 => {
                match k {
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
                let vx = self.v[x];
                let vy = self.v[y];
                self.skip_if_not_equal(vx, vy);
            },
            0xA000 => {
                self.load_i(nnn);
            },
            0xB000 => {
                let v0 = self.v[0];
                self.jump(nnn + (v0 as Address));
            },
            0xC000 => {
                self.random_and(x, kk);
            },
            0xD000 => {
                self.draw_sprite(x, y, k as usize);
            },
            0xE000 => {
                match kk {
                    0x9E => (), // skip if key[v[x]] down
                    0xA1 => (), // skip if key[v[x]] up
                    _ => ()
                }
            },
            0xF000 => {
                match kk {
                    0x07 => {
                        let dt = self.dt.current();
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

    fn set_flag(&mut self, val: Byte) {
        self.v[0xF] = val;
    }

    fn clear_display(&mut self) {
        self.memory.clear(&Memory::DISPLAY_RANGE)
    }

    fn return_from_subroutine(&mut self) {
        if self.sp < Memory::STACK_RANGE.start { return; }

        self.pc = self.memory.stack_pop(&self.sp);
        self.sp -= 2;
    }

    fn jump(&mut self, addr: Address) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: Address) {
        if self.sp + 2 >= Memory::STACK_RANGE.end { return; }

        self.sp += 2;
        self.memory.stack_push(&self.sp, &self.pc);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_00e0_clear_display() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        for i in Memory::DISPLAY_RANGE {
            cpu.memory[i] = 0xFF;
            assert_eq!(0xFF, cpu.memory[i]);
        }

        cpu.execute(0x00E0);
        for i in Memory::DISPLAY_RANGE {
            assert_eq!(0x0, cpu.memory[i]);
        }
    }

    #[test]
    fn opcode_00ee_return_from_subroutine() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.memory[Memory::STACK_RANGE.start] = 0x03;
        cpu.memory[Memory::STACK_RANGE.start + 1] = 0x45;
        cpu.sp += 2;

        cpu.execute(0x00EE);
        assert_eq!(Memory::STACK_RANGE.start - 2, cpu.sp);
        assert_eq!(0x0345, cpu.pc);
    }

    #[test]
    fn opcode_1nnn_jump() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.execute(0x1234);
        assert_eq!(0x234, cpu.pc);
    }

    #[test]
    fn opcode_2nnn_call_subroutine() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);
        cpu.pc = 0x234;

        cpu.execute(0x2456);
        assert_eq!(0x02, cpu.memory[Memory::STACK_RANGE.start]);
        assert_eq!(0x34, cpu.memory[Memory::STACK_RANGE.start + 1]);
        assert_eq!(Memory::STACK_RANGE.start, cpu.sp);
        assert_eq!(0x456, cpu.pc);
    }

    #[test]
    fn opcode_3xkk_skip_if_equal() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let pc = cpu.pc;
        let x = 0xA;
        let kk = 0xBC;

        cpu.v[x] = kk + 1;
        cpu.execute(0x3ABC);
        assert_eq!(pc, cpu.pc);

        cpu.v[x] = kk;
        cpu.execute(0x3ABC);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn opcode_4xkk_skip_if_not_equal() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let pc = cpu.pc;
        let x = 0xA;
        let kk = 0xBC;

        cpu.v[x] = kk;
        cpu.execute(0x4ABC);
        assert_eq!(pc, cpu.pc);

        cpu.v[x] = kk + 1;
        cpu.execute(0x4ABC);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn opcode_5xy0_skip_if_equal() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let pc = cpu.pc;
        let x = 0xA;
        let y = 0xB;
        cpu.v[x] = 0xBC;

        cpu.v[y] = cpu.v[x] + 1;
        cpu.execute(0x5AB0);
        assert_eq!(pc, cpu.pc);

        cpu.v[y] = cpu.v[x];
        cpu.execute(0x5AB0);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn opcode_6xkk_load() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.execute(0x6ABC);
        assert_eq!(0xBC, cpu.v[0xA]);
    }

    #[test]
    fn opcode_7xkk_add() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.execute(0x7AFF);
        assert_eq!(0xFF, cpu.v[0xA]);

        cpu.execute(0x7AFF);
        assert_eq!(0xFE, cpu.v[0xA]);
    }

    #[test]
    fn opcode_8xy1_or() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0b1;
        cpu.v[y] = 0b0;
        cpu.execute(0x8AB1);
        assert_eq!(0b1, cpu.v[x]);

        cpu.v[x] = 0b0;
        cpu.v[y] = 0b1;
        cpu.execute(0x8AB1);
        assert_eq!(0b1, cpu.v[x]);

        cpu.v[x] = 0b0;
        cpu.v[y] = 0b0;
        cpu.execute(0x8AB1);
        assert_eq!(0b0, cpu.v[x]);

        cpu.v[x] = 0b1;
        cpu.v[y] = 0b1;
        cpu.execute(0x8AB1);
        assert_eq!(0b1, cpu.v[x]);
    }

    #[test]
    fn opcode_8xy2_and() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0b1;
        cpu.v[y] = 0b0;
        cpu.execute(0x8AB2);
        assert_eq!(0b0, cpu.v[x]);

        cpu.v[x] = 0b0;
        cpu.v[y] = 0b1;
        cpu.execute(0x8AB2);
        assert_eq!(0b0, cpu.v[x]);

        cpu.v[x] = 0b0;
        cpu.v[y] = 0b0;
        cpu.execute(0x8AB2);
        assert_eq!(0b0, cpu.v[x]);

        cpu.v[x] = 0b1;
        cpu.v[y] = 0b1;
        cpu.execute(0x8AB2);
        assert_eq!(0b1, cpu.v[x]);
    }

    #[test]
    fn opcode_8xy3_xor() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0b1;
        cpu.v[y] = 0b0;
        cpu.execute(0x8AB3);
        assert_eq!(0b1, cpu.v[x]);

        cpu.v[x] = 0b0;
        cpu.v[y] = 0b1;
        cpu.execute(0x8AB3);
        assert_eq!(0b1, cpu.v[x]);

        cpu.v[x] = 0b0;
        cpu.v[y] = 0b0;
        cpu.execute(0x8AB3);
        assert_eq!(0b0, cpu.v[x]);

        cpu.v[x] = 0b1;
        cpu.v[y] = 0b1;
        cpu.execute(0x8AB3);
        assert_eq!(0b0, cpu.v[x]);
    }

    #[test]
    fn opcode_8xy4_add_with_carry() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0x1;
        cpu.v[y] = 0xFE;
        cpu.execute(0x8AB4);
        assert_eq!(0xFF, cpu.v[x]);
        assert_eq!(0b0, cpu.v[0xF]);

        cpu.v[x] = 0x1;
        cpu.v[y] = 0xFF;
        cpu.execute(0x8AB4);
        assert_eq!(0x0, cpu.v[x]);
        assert_eq!(0b1, cpu.v[0xF]);
    }

    #[test]
    fn opcode_8xy5_subtract_without_borrow() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0x0;
        cpu.v[y] = 0xFF;
        cpu.execute(0x8AB5);
        assert_eq!(0x1, cpu.v[x]);
        assert_eq!(0b0, cpu.v[0xF]);

        cpu.v[x] = 0x1;
        cpu.v[y] = 0x1;
        cpu.execute(0x8AB5);
        assert_eq!(0x0, cpu.v[x]);
        assert_eq!(0b1, cpu.v[0xF]);
    }

    #[test]
    fn opcode_8xy6_shift_right() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;

        cpu.v[x] = 0b11;
        cpu.execute(0x8A06);
        assert_eq!(0b1, cpu.v[x]);
        assert_eq!(0b1, cpu.v[0xF]);

        cpu.v[x] = 0b10;
        cpu.execute(0x8A06);
        assert_eq!(0b1, cpu.v[x]);
        assert_eq!(0b0, cpu.v[0xF]);
    }

    #[test]
    fn opcode_8xy7_subtract_neg_without_borrow() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0xFF;
        cpu.v[y] = 0x0;
        cpu.execute(0x8AB7);
        assert_eq!(0x1, cpu.v[x]);
        assert_eq!(0b0, cpu.v[0xF]);

        cpu.v[x] = 0x1;
        cpu.v[y] = 0x1;
        cpu.execute(0x8AB7);
        assert_eq!(0x0, cpu.v[x]);
        assert_eq!(0b1, cpu.v[0xF]);
    }

    #[test]
    fn opcode_8xye_shift_left() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let x = 0xA;

        cpu.v[x] = 0b11;
        cpu.execute(0x8A0E);
        assert_eq!(0b110, cpu.v[x]);
        assert_eq!(0b0, cpu.v[0xF]);

        cpu.v[x] = 0b10000001;
        cpu.execute(0x8A0E);
        assert_eq!(0b10, cpu.v[x]);
        assert_eq!(0b1, cpu.v[0xF]);
    }

    #[test]
    fn opcode_9xy0_skip_if_not_equal() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        let pc = cpu.pc;
        let x = 0xA;
        let y = 0xB;

        cpu.v[x] = 0xBC;
        cpu.v[y] = cpu.v[x];
        cpu.execute(0x9AB0);
        assert_eq!(pc, cpu.pc);

        cpu.v[y] = cpu.v[x] + 1;
        cpu.execute(0x9AB0);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn opcode_annn_load_i() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.execute(0xABCD);
        assert_eq!(0xBCD, cpu.i);
    }

    #[test]
    fn opcode_bnnn_jump_plus_v0() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.v[0] = 0x2;
        cpu.execute(0xBBCD);
        assert_eq!(0xBCF, cpu.pc);
    }

    #[test]
    fn opcode_fx07_set_vx_dt() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.dt.set(0x42);
        cpu.execute(0xFA07);
        assert_eq!(0x42, cpu.v[0xA]);
    }

    #[test]
    fn opcode_fx15_set_dt_vx() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.v[0xA] = 0x66;
        cpu.execute(0xFA15);
        assert_eq!(0x66, cpu.dt.current());
    }

    #[test]
    fn opcode_fx18_set_st_vx() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.v[0xA] = 0x66;
        cpu.execute(0xFA18);
        assert_eq!(0x66, cpu.st.current());
    }

    #[test]
    fn opcode_fx1e_set_i_plus_vx() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.i = 0xC00;
        cpu.v[0xA] = 0x2;
        cpu.execute(0xFA1E);
        assert_eq!(0xC02, cpu.i);
    }

    #[test]
    fn opcode_fx29_set_i_sprite_loc_vx() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        for sprite in 0..0xF {
            cpu.v[0xA] = sprite;
            cpu.execute(0xFA29);
            assert_eq!(font::sprite_addr(sprite), cpu.i);
        }
    }

    #[test]
    fn opcode_fx33_store_bcd() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.v[0xA] = 0xFE;
        cpu.execute(0xFA33);
        assert_eq!(0x2, cpu.memory[cpu.i]);
        assert_eq!(0x5, cpu.memory[cpu.i + 1]);
        assert_eq!(0x4, cpu.memory[cpu.i + 2]);
    }

    #[test]
    fn opcode_fx55_store_registers_through() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.v[0x0] = 0x10;
        cpu.v[0x1] = 0x11;
        cpu.v[0x2] = 0x12;
        cpu.v[0x3] = 0x13;
        cpu.execute(0xF355);
        assert_eq!(0x10, cpu.memory[cpu.i]);
        assert_eq!(0x11, cpu.memory[cpu.i + 1]);
        assert_eq!(0x12, cpu.memory[cpu.i + 2]);
        assert_eq!(0x13, cpu.memory[cpu.i + 3]);
    }

    #[test]
    fn opcode_fx65_read_registers_through() {
        let rom = Vec::new();
        let mut mem = Memory::new(&rom);
        let mut cpu = Cpu::new(&mut mem);

        cpu.memory[cpu.i] = 0xFF;
        cpu.memory[cpu.i + 1] = 0xFE;
        cpu.memory[cpu.i + 2] = 0xFD;
        cpu.memory[cpu.i + 3] = 0xFC;
        cpu.execute(0xF365);
        assert_eq!(0xFF, cpu.v[0x0]);
        assert_eq!(0xFE, cpu.v[0x1]);
        assert_eq!(0xFD, cpu.v[0x2]);
        assert_eq!(0xFC, cpu.v[0x3]);
    }
}
