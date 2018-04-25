const NUM_REGISTERS: usize = 16;

const MAX_ADDR: Address = 0x1000;
const FONT_RANGE: Range<Address> = 0x0..0x200;
const ROM_RANGE: Range<Address> = 0x200..0xFA0;
const STACK_RANGE: Range<Address> = 0xFA0..MAX_ADDR;

extern crate rand;

mod opcode;
mod ops;
mod timer;

use self::rand::Rng;
use std::ops::Range;

use cpu::ops::Operation;
use cpu::opcode::Opcode;
use cpu::timer::Timer;
use memory::Memory;
use output::{font, graphics};

use {Address, Byte};
type Register = usize;

#[derive(Debug, PartialEq)]
pub struct Cpu<'a, G: 'a> where G: graphics::GraphicsOutput {
    pub exit: bool,
    pub beep: bool,
    pc: Address,
    sp: Address,
    i: Address,
    dt: Timer,
    st: Timer,
    v: [Byte; NUM_REGISTERS],
    memory: Memory<Byte>,
    graphics: &'a mut G
}

impl<'a, G> Cpu<'a, G> where G: graphics::GraphicsOutput {
    pub fn new(rom: &Vec<Byte>, graphics: &'a mut G) -> Cpu<'a, G> {
        let mut memory = Memory::new(MAX_ADDR, 0x0);
        memory.load(&font::FONT_SET, FONT_RANGE);
        memory.load(&rom, ROM_RANGE);

        Cpu {
            exit: false,
            beep: true,
            pc: ROM_RANGE.start,
            sp: STACK_RANGE.start,
            i: FONT_RANGE.start,
            dt: Timer::new(60, 60),
            st: Timer::new(60, 60),
            v: [0x0; NUM_REGISTERS],
            memory,
            graphics
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_opcode();
        let op = self.operation(&opcode);
        op(self, &opcode);

        self.update_timers();
    }

    pub fn fetch_opcode(&self) -> Opcode {
        let current = self.pc;
        let bytes = (self.memory[current], self.memory[current + 1]);
        Opcode::from_bytes(bytes)
    }

    pub fn operation(&mut self, opcode: &Opcode) -> fn(&mut Cpu<'a, G>, &Opcode) {
        match opcode.first_hex_digit() {
            0x0 => {
                match opcode.kk() {
                    0x00 => Cpu::no_op,
                    0xE0 => Cpu::clear_display,
                    0xEE => Cpu::return_from_subroutine,
                    _ => Cpu::unknown
                }
            },
            0x1 => Cpu::jump_addr,
            0x2 => Cpu::call_addr,
            0x3 => Cpu::skip_equal_vx_byte,
            0x4 => Cpu::skip_not_equal_vx_byte,
            0x5 => Cpu::skip_equal_vx_vy,
            0x6 => Cpu::load_vx_byte,
            0x7 => Cpu::add_vx_byte,
            0x8 => {
                match opcode.k() {
                    0x0 => Cpu::load_vx_vy,
                    0x1 => Cpu::or_vx_vy,
                    0x2 => Cpu::and_vx_vy,
                    0x3 => Cpu::xor_vx_vy,
                    0x4 => Cpu::add_vx_vy,
                    0x5 => Cpu::sub_vx_vy,
                    0x6 => Cpu::shr_vx_vy,
                    0x7 => Cpu::subn_vx_vy,
                    0xE => Cpu::shl_vx_vy,
                    _ => Cpu::unknown
                }
            },
            0x9 => Cpu::skip_not_equal_vx_vy,
            0xA => Cpu::load_i_addr,
            0xB => Cpu::jump_v0_addr,
            0xC => Cpu::rand_vx_byte,
            0xD => Cpu::draw_vx_vy_n,
            0xE => {
                match opcode.kk() {
                    0x9E => Cpu::skip_key_pressed_vx,
                    0xA1 => Cpu::skip_key_not_pressed_vx,
                    _ => Cpu::unknown
                }
            },
            0xF => {
                match opcode.kk() {
                    0x07 => Cpu::load_vx_dt,
                    0x0A => Cpu::load_vx_key,
                    0x15 => Cpu::load_dt_vx,
                    0x18 => Cpu::load_st_vx,
                    0x1E => Cpu::add_i_vx,
                    0x29 => Cpu::load_i_vx_font,
                    0x33 => Cpu::load_bcd_vx,
                    0x55 => Cpu::load_through_vx,
                    0x65 => Cpu::read_through_vx,
                    _ => Cpu::unknown
                }
            },
            _ => Cpu::unknown
        }
    }

    fn advance_pc(&mut self) {
        self.pc += 2;
    }

    fn advance_sp(&mut self) {
        self.sp += 2;
    }

    fn retract_sp(&mut self) {
        self.sp -= 2;
    }

    fn read_i(&self) -> Address {
        self.i
    }

    fn load_i(&mut self, addr: Address) {
        self.i = addr;
    }

    fn load_register(&mut self, register: Register, val: Byte) {
        self.v[register] = val;
    }

    fn read_register(&self, register: Register) -> Byte {
        self.v[register]
    }

    fn load_flag(&mut self, b: bool) {
        if b {
            self.load_register(0xF, 0b1);
        } else {
            self.load_register(0xF, 0b0);
        }
    }

    fn read_bytes(&self, addr: Address, n: usize) -> Vec<Byte> {
        self.memory[addr..addr + n].to_vec()
    }

    fn load_byte(&mut self, addr: Address, byte: Byte) {
        self.memory[addr] = byte;
    }

    fn draw_byte(&mut self, x: Address, y: Address, byte: Byte) -> bool {
        let mut collision = false;
        for b in 0..8 {
            let bit = byte.wrapping_shr(8 - b - 1) & 0b1;
            if self.graphics.update_pixel(x + b as usize, y, bit == 1) {
                collision = true;
            }
        }
        collision
    }

    fn read_delay_timer(&self) -> Byte {
        self.dt.current
    }

    fn load_delay_timer(&mut self, val: Byte) {
        self.dt.set(val);
    }

    fn read_sound_timer(&mut self) -> Byte {
        self.st.current
    }

    fn load_sound_timer(&mut self, val: Byte) {
        self.st.set(val);
    }

    fn update_timers(&mut self) {
        self.dt.tick();
        self.st.tick();
        self.beep = self.st.active();
    }

    fn stack_pop(&mut self) -> Address {
        let current = self.sp;
        let addr = (self.memory[current] as Address) << 8 | (self.memory[current + 1] as Address);
        self.retract_sp();
        addr
    }

    fn stack_push(&mut self) {
        self.advance_sp();
        let current = self.sp;
        let addr = self.pc;
        self.memory[current] = ((addr & 0xFF00) >> 8) as Byte;
        self.memory[current + 1] = (addr & 0x00FF) as Byte;
    }
}

impl<'a, G> Operation for Cpu<'a, G> where G: graphics::GraphicsOutput {
    fn no_op(&mut self, _opcode: &Opcode) {
        self.advance_pc();
    }

    fn unknown(&mut self, opcode: &Opcode) {
        panic!("Unknown opcode {}", opcode);
    }

    fn clear_display(&mut self, _opcode: &Opcode) {
        self.graphics.clear();
        self.advance_pc();
        println!("\tCLS");
    }

    fn return_from_subroutine(&mut self, _opcode: &Opcode) {
        let addr = self.stack_pop();
        self.pc = addr;
        self.advance_pc();
        println!("\tRTN => {:x}", addr);
    }

    fn jump_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();

        if self.pc == addr {
            self.exit = true;
            return;
        }

        self.pc = addr;
        println!("\tJP {:x}", addr);
    }

    fn call_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.stack_push();
        self.pc = addr;
        println!("\tCALL {:x}", addr);
    }

    fn skip_equal_vx_byte(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let byte = opcode.kk();
        if vx == byte { self.advance_pc(); }
        self.advance_pc();
        println!("\tSE vx: {:x}, byte: {:x}", vx, byte);
    }

    fn skip_not_equal_vx_byte(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let byte = opcode.kk();
        if vx != byte { self.advance_pc(); }
        self.advance_pc();
        println!("\tSNE vx: {:x}, byte: {:x}", vx, byte);
    }

    fn skip_equal_vx_vy(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let vy = self.read_register(opcode.y());
        if vx == vy { self.advance_pc(); }
        self.advance_pc();
        println!("\tSE vx: {:x}, vy: {:x}", vx, vy);
    }

    fn load_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        self.load_register(x, opcode.kk());
        self.advance_pc();
        println!("\tLD V{:x}, byte: {:x} => {:x}", x, opcode.kk(), self.read_register(x));
    }

    fn add_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let byte = opcode.kk();
        let vx = self.read_register(x);
        self.load_register(x, vx.wrapping_add(byte));
        self.advance_pc();
        println!("\tADD V{:x}: {:x}, byte: {:x} => {:x}", x, vx, byte, self.read_register(x));
    }

    fn load_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        self.load_register(x, vy);
        self.advance_pc();
        println!("\tLD V{:x}, V{:x}: {:x} => {:x}", x, y, vy, self.read_register(x));
    }

    fn or_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx | vy);
        self.advance_pc();
        println!("\tOR V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn and_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx & vy);
        self.advance_pc();
        println!("\tAND V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn xor_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx ^ vy);
        self.advance_pc();
        println!("\tXOR V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn add_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        let (result, carry) = vx.overflowing_add(vy);
        self.load_register(x, result);
        self.load_flag(carry);
        self.advance_pc();
        println!("\tADD V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn sub_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        let (result, borrow) = vx.overflowing_sub(vy);
        self.load_register(x, result);
        self.load_flag(!borrow);
        self.advance_pc();
        println!("\tSUB V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn shr_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        let bit = vy % 2;
        self.load_register(x, vy.wrapping_shr(1));
        self.load_flag(bit == 1);
        self.advance_pc();
        println!("\tSHR V{:x}, V{:x}: {:x} => ({:x}, {})", x, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn subn_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        let (result, borrow) = vy.overflowing_sub(vx);
        self.load_register(x, result);
        self.load_flag(!borrow);
        self.advance_pc();
        println!("\tSUBN V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn shl_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        let bit = vy >> 7;
        self.load_register(x, vy.wrapping_shl(1));
        self.load_flag(bit == 1);
        self.advance_pc();
        println!("\tSHL V{:x}, V{:x}: {:x} => ({:x}, {})", x, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn skip_not_equal_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        if vx != vy { self.advance_pc(); }
        self.advance_pc();
        println!("\tSNE V{:x}: {:x}, V{:x}: {:x}", x, vx, y, vy);
    }

    fn load_i_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.load_i(addr);
        self.advance_pc();
        println!("\tLD I, {:x} => {:x}", addr, self.read_i());
    }

    fn jump_v0_addr(&mut self, opcode: &Opcode) {
        let v0 = self.read_register(0x0) as Address;
        let addr = opcode.nnn();
        self.pc = addr + v0;
        println!("\tJP V0: {:x}, {:x}", v0, addr);
    }

    fn rand_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let byte = opcode.kk();
        let random_byte: Byte = rand::thread_rng().gen_range(0x0, 0xFF);
        self.load_register(x, random_byte & byte);
        self.advance_pc();
        println!("\tRND V{:x} => {:x}", x, self.read_register(x));
    }

    fn draw_vx_vy_n(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x()) as Address;
        let vy = self.read_register(opcode.y()) as Address;
        let n = opcode.k();
        let i = self.read_i();
        let sprite_bytes = self.read_bytes(i, n);
        let mut collision = false;
        for sprite_y in 0..n {
            let sprite_byte = sprite_bytes[sprite_y];
            if self.draw_byte(vx, vy + sprite_y, sprite_byte) {
                collision = true;
            }
        }

        self.load_flag(collision);
        self.graphics.draw();
        self.advance_pc();

        println!("\tDRW Vx: {:x}, Vy: {:x}, {:?}", vx, vy, sprite_bytes);
    }

    fn skip_key_pressed_vx(&mut self, _opcode: &Opcode) {}
    fn skip_key_not_pressed_vx(&mut self, _opcode: &Opcode) {}

    fn load_vx_dt(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let dt = self.read_delay_timer();
        self.load_register(x, dt);
        self.advance_pc();
        println!("\tLD V{:x}, DT: {:x} => {:x}", x, dt, self.read_register(x));
    }

    fn load_vx_key(&mut self, _opcode: &Opcode) {}

    fn load_dt_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        self.load_delay_timer(vx);
        self.advance_pc();
        println!("\tLD DT, V{:x}: {:x} => {:x}", x, vx, self.read_delay_timer());
    }

    fn load_st_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        self.load_sound_timer(vx);
        self.advance_pc();
        println!("\tLD ST, V{:x}: {:x} => {:x}", x, vx, self.read_sound_timer());
    }

    fn add_i_vx(&mut self, opcode: &Opcode) {
        let i = self.read_i();
        let x = opcode.x();
        let vx = self.read_register(x) as Address;
        self.load_i(i.wrapping_add(vx));
        self.advance_pc();
        println!("\tADD I: {:x}, V{:x}: {:x} => {:x}", i, x, vx, self.read_i());
    }

    fn load_i_vx_font(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x) as Address;
        self.load_i(vx * font::SPRITE_HEIGHT);
        self.advance_pc();
        println!("\tLD I, FONT V{:x}: {:x} => {:x}", x, vx, self.read_i());
    }

    fn load_bcd_vx(&mut self, opcode: &Opcode) {
        let i = self.read_i();
        let x = opcode.x();
        let vx = self.read_register(x);
        self.memory[i + 0] = vx / 100;
        self.memory[i + 1] = vx % 100 / 10;
        self.memory[i + 2] = vx % 10;
        self.advance_pc();
        println!("\tLD BCD V{:x}: {:x} ({}) => {:?}", x, vx, vx, self.memory[i..i + 2].to_vec());
    }

    fn load_through_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let i = self.read_i();
        let bytes: Vec<Byte> = (0..x + 1).map(|r| self.read_register(r)).collect();
        for (addr, byte) in bytes.iter().enumerate() {
            self.load_byte(i + addr, *byte);
        }
        self.load_i(i + x + 1);
        self.advance_pc();

        let i = self.read_i();
        let register_bytes: Vec<Byte> = (0..x + 1).map(|r| self.read_register(r)).collect();
        println!("\tLD [I], V{:x} [{:?}] => {:?}", x, register_bytes, self.read_bytes(i, x + 1));
    }

    fn read_through_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let i = self.read_i();
        let bytes = self.read_bytes(i, x + 1);
        for (r, byte) in bytes.iter().enumerate() {
            self.load_register(r, *byte);
        }
        self.load_i(i + x + 1);
        self.advance_pc();

        let i = self.read_i();
        let register_bytes: Vec<Byte> = (0..x + 1).map(|r| self.read_register(r)).collect();
        println!("\tRD V{:x} [{:?}], [I] => {:?}", x, self.read_bytes(i, x + 1), register_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use output::graphics::GraphicsOutput;

    #[test]
    fn new_loads_font_to_memory() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let cpu = Cpu::new(&rom, &mut graphics);

        let font_set = font::FONT_SET.to_vec();
        let range = FONT_RANGE.start..FONT_RANGE.start + font::FONT_SET.len();
        let mem = cpu.memory[range].to_vec();
        assert_eq!(font_set, mem);
    }

    #[test]
    fn new_loads_rom_to_memory() {
        let mut graphics = graphics::Display::new();
        let rom = vec![0x00, 0x01, 0x02, 0x03];
        let cpu = Cpu::new(&rom, &mut graphics);
        
        let range = ROM_RANGE.start..ROM_RANGE.start + rom.len();
        let mem = cpu.memory[range].to_vec();
        assert_eq!(rom, mem);
    }

    #[test]
    fn fetch_opcode_fetches_two_current_bytes() {
        let mut graphics = graphics::Display::new();
        let rom = vec![0xAB, 0xCD, 0xEF, 0xFF];
        let cpu = Cpu::new(&rom, &mut graphics);

        assert_eq!(Opcode::new(0xABCD), cpu.fetch_opcode());        
    }

    #[test]
    fn operation_0000_no_op() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x0000);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;
        let memory = cpu.memory.clone();

        op(&mut cpu, &opcode);
        assert_eq!(memory, cpu.memory);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_00e0_clear_display() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x00E0);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        op(&mut cpu, &opcode);
        for x in 0..graphics::SCREEN_WIDTH {
            for y in 0..graphics::SCREEN_HEIGHT {
                assert!(!cpu.graphics.read_pixel(x, y));
            }
        }
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_00ee_return_from_subroutine() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x00EE);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.stack_push();
        cpu.advance_pc();
        cpu.advance_pc();
        cpu.advance_pc();

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_1nnn_jump_addr() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x1404);
        let op = cpu.operation(&opcode);

        op(&mut cpu, &opcode);
        assert_eq!(0x404, cpu.pc);
    }

    #[test]
    fn operation_2nnn_call_subroutine() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x2404);
        let op = cpu.operation(&opcode);

        op(&mut cpu, &opcode);
        assert_eq!(0x404, cpu.pc);
        assert_eq!(0x02, cpu.memory[cpu.sp]);
        assert_eq!(0x00, cpu.memory[cpu.sp + 1]);
    }

    #[test]
    fn operation_3xkk_skip_equal_vx_byte() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x3123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc);
    }

    #[test]
    fn operation_3xkk_skip_equal_vx_byte_no_skip() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x3122);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;
        
        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_4xkk_skip_not_equal_vx_byte() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x4123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0x22;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc);
    }

    #[test]
    fn operation_4xkk_skip_not_equal_vx_byte_no_skip() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x4123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_5xy0_skip_equal_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x5010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = cpu.v[0x0];

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc);
    }

    #[test]
    fn operation_5xy0_skip_equal_vx_vy_no_skip() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x5010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = 0x22;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_6xkk_load_vx_byte() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x6123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        op(&mut cpu, &opcode);
        assert_eq!(0x23, cpu.read_register(0x1));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_7xkk_add_vx_byte() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x71FF);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x1));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_7xkk_add_vx_byte_wrap() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x71FF);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x1));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy0_load_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(0x23, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_00() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_01() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_10() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_11() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_00() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_01() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_10() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_11() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_00() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_01() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_10() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_11() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy4_add_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8014);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x03, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy4_add_vx_vy_carry() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8014);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0xFF;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy5_sub_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8015);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x02;
        cpu.v[0x1] = 0x01;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy5_sub_vx_vy_carry() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8015);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy6_shr_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8016);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0b11111110;

        op(&mut cpu, &opcode);
        assert_eq!(0b01111111, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy6_shr_vx_vy_sig_bit() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8016);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0b11111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b01111111, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy7_subn_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8017);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xy7_subn_vx_vy_carry() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x8017);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x02;
        cpu.v[0x1] = 0x01;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xye_shl_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x801E);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0b01111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b11111110, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_8xye_shl_vx_vy_sig_bit() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x801E);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x1] = 0b11111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b11111110, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));   
        assert_eq!(pc + 2, cpu.pc);     
    }

    #[test]
    fn operation_9xy0_skip_not_equal_vx_vy() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x9010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = 0x22;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc);
    }

    #[test]
    fn operation_9xy0_skip_not_equal_vx_vy_no_skip() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0x9010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = cpu.v[0x0];

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_annn_load_i_addr() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xA456);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        op(&mut cpu, &opcode);
        assert_eq!(0x456, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_bnnn_jump_v0_addr() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xB300);
        let op = cpu.operation(&opcode);

        cpu.v[0x0] = 0x08;

        op(&mut cpu, &opcode);
        assert_eq!(0x308, cpu.pc);
    }

    #[test]
    fn operation_cxkk_rand_vx_byte() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xC00F);
        let op = cpu.operation(&opcode);

        for _ in 0..1000 {
            let pc = cpu.pc;

            op(&mut cpu, &opcode);
            assert!(cpu.read_register(0x0) <= 0xF);
            assert_eq!(pc + 2, cpu.pc);
        }
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 8;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b11110000;
        cpu.memory[i + 1] = 0b00001111;

        op(&mut cpu, &opcode);
        assert!(cpu.graphics.read_pixel(8, 5));
        assert!(cpu.graphics.read_pixel(9, 5));
        assert!(cpu.graphics.read_pixel(10, 5));
        assert!(cpu.graphics.read_pixel(11, 5));
        assert!(!cpu.graphics.read_pixel(12, 5));
        assert!(!cpu.graphics.read_pixel(13, 5));
        assert!(!cpu.graphics.read_pixel(14, 5));
        assert!(!cpu.graphics.read_pixel(15, 5));

        assert!(!cpu.graphics.read_pixel(8, 6));
        assert!(!cpu.graphics.read_pixel(9, 6));
        assert!(!cpu.graphics.read_pixel(10, 6));
        assert!(!cpu.graphics.read_pixel(11, 6));
        assert!(cpu.graphics.read_pixel(12, 6));
        assert!(cpu.graphics.read_pixel(13, 6));
        assert!(cpu.graphics.read_pixel(14, 6));
        assert!(cpu.graphics.read_pixel(15, 6));

        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_collision() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 8;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b11110000;
        cpu.memory[i + 1] = 0b00001111;

        cpu.graphics.update_pixel(12, 6, true);

        op(&mut cpu, &opcode);
        assert!(cpu.graphics.read_pixel(8, 5));
        assert!(cpu.graphics.read_pixel(9, 5));
        assert!(cpu.graphics.read_pixel(10, 5));
        assert!(cpu.graphics.read_pixel(11, 5));
        assert!(!cpu.graphics.read_pixel(12, 5));
        assert!(!cpu.graphics.read_pixel(13, 5));
        assert!(!cpu.graphics.read_pixel(14, 5));
        assert!(!cpu.graphics.read_pixel(15, 5));

        assert!(!cpu.graphics.read_pixel(8, 6));
        assert!(!cpu.graphics.read_pixel(9, 6));
        assert!(!cpu.graphics.read_pixel(10, 6));
        assert!(!cpu.graphics.read_pixel(11, 6));
        assert!(!cpu.graphics.read_pixel(12, 6));
        assert!(cpu.graphics.read_pixel(13, 6));
        assert!(cpu.graphics.read_pixel(14, 6));
        assert!(cpu.graphics.read_pixel(15, 6));

        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vx_wrap() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 60;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b11110000;
        cpu.memory[i + 1] = 0b00001111;

        op(&mut cpu, &opcode);
        assert!(cpu.graphics.read_pixel(60, 5));
        assert!(cpu.graphics.read_pixel(61, 5));
        assert!(cpu.graphics.read_pixel(62, 5));
        assert!(cpu.graphics.read_pixel(63, 5));
        assert!(!cpu.graphics.read_pixel(0, 5));
        assert!(!cpu.graphics.read_pixel(1, 5));
        assert!(!cpu.graphics.read_pixel(2, 5));
        assert!(!cpu.graphics.read_pixel(3, 5));

        assert!(!cpu.graphics.read_pixel(60, 6));
        assert!(!cpu.graphics.read_pixel(61, 6));
        assert!(!cpu.graphics.read_pixel(62, 6));
        assert!(!cpu.graphics.read_pixel(63, 6));
        assert!(cpu.graphics.read_pixel(0, 6));
        assert!(cpu.graphics.read_pixel(1, 6));
        assert!(cpu.graphics.read_pixel(2, 6));
        assert!(cpu.graphics.read_pixel(3, 6));

        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vy_wrap() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 8;
        cpu.v[1] = 31;
        cpu.memory[i + 0] = 0b11110000;
        cpu.memory[i + 1] = 0b00001111;

        op(&mut cpu, &opcode);
        assert!(cpu.graphics.read_pixel(8, 31));
        assert!(cpu.graphics.read_pixel(9, 31));
        assert!(cpu.graphics.read_pixel(10, 31));
        assert!(cpu.graphics.read_pixel(11, 31));
        assert!(!cpu.graphics.read_pixel(12, 31));
        assert!(!cpu.graphics.read_pixel(13, 31));
        assert!(!cpu.graphics.read_pixel(14, 31));
        assert!(!cpu.graphics.read_pixel(15, 31));

        assert!(!cpu.graphics.read_pixel(8, 0));
        assert!(!cpu.graphics.read_pixel(9, 0));
        assert!(!cpu.graphics.read_pixel(10, 0));
        assert!(!cpu.graphics.read_pixel(11, 0));
        assert!(cpu.graphics.read_pixel(12, 0));
        assert!(cpu.graphics.read_pixel(13, 0));
        assert!(cpu.graphics.read_pixel(14, 0));
        assert!(cpu.graphics.read_pixel(15, 0));

        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx07_load_vx_dt() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF007);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        let time = 0x20;
        cpu.dt.set(time);

        op(&mut cpu, &opcode);
        assert_eq!(time, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx15_load_dt_vx() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF015);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        let time = 0x20;
        cpu.v[0x0] = time;

        op(&mut cpu, &opcode);
        assert_eq!(time, cpu.read_delay_timer());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx18_load_st_vx() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF018);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        let time = 0x20;
        cpu.v[0x0] = time;

        op(&mut cpu, &opcode);
        assert_eq!(time, cpu.read_sound_timer());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx1e_add_i_vx() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF01E);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;
        let i = cpu.read_i();

        cpu.v[0x0] = 0x08;

        op(&mut cpu, &opcode);
        assert_eq!(i + 0x08, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx29_load_i_vx_font() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF029);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.v[0x0] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(FONT_RANGE.start + 10, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx33_load_bcd_vx() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF033);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;
        let i = cpu.read_i();

        cpu.v[0x0] = 0xFE;

        op(&mut cpu, &opcode);
        assert_eq!(0x02, cpu.memory[i + 0]);
        assert_eq!(0x05, cpu.memory[i + 1]);
        assert_eq!(0x04, cpu.memory[i + 2]);
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx55_load_through_vx() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF555);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 0xFF;
        cpu.v[1] = 0xEE;
        cpu.v[2] = 0xDD;
        cpu.v[3] = 0xCC;
        cpu.v[4] = 0xBB;
        cpu.v[5] = 0xAA;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.memory[i + 0]);
        assert_eq!(0xEE, cpu.memory[i + 1]);
        assert_eq!(0xDD, cpu.memory[i + 2]);
        assert_eq!(0xCC, cpu.memory[i + 3]);
        assert_eq!(0xBB, cpu.memory[i + 4]);
        assert_eq!(0xAA, cpu.memory[i + 5]);
        assert_eq!(i + 6, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn operation_fx65_read_through_vx() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        let opcode = Opcode::new(0xF565);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.memory[i + 0] = 0xFF;
        cpu.memory[i + 1] = 0xEE;
        cpu.memory[i + 2] = 0xDD;
        cpu.memory[i + 3] = 0xCC;
        cpu.memory[i + 4] = 0xBB;
        cpu.memory[i + 5] = 0xAA;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x0));
        assert_eq!(0xEE, cpu.read_register(0x1));
        assert_eq!(0xDD, cpu.read_register(0x2));
        assert_eq!(0xCC, cpu.read_register(0x3));
        assert_eq!(0xBB, cpu.read_register(0x4));
        assert_eq!(0xAA, cpu.read_register(0x5));
        assert_eq!(i + 6, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc);
    }

    #[test]
    fn beep_while_sound_timer_active() {
        let mut graphics = graphics::Display::new();
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom, &mut graphics);

        cpu.update_timers();

        assert!(cpu.st.active());
        assert!(cpu.beep);

        cpu.st.set(0);
        cpu.update_timers();

        assert!(!cpu.st.active());
        assert!(!cpu.beep);
    }
}
