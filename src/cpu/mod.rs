const NUM_REGISTERS: usize = 16;

const FONT_RANGE: Range<Address> = 0x0..0x200;
const ROM_RANGE: Range<Address> = 0x200..0xEA0;
const STACK_RANGE: Range<Address> = 0xEA0..0xF00;
const DISPLAY_RANGE: Range<Address> = 0xF00..Memory::MAX_SIZE;

extern crate rand;

mod opcode;
mod ops;
mod pointer;
mod timer;

use self::rand::Rng;
use std::ops::Range;

use cpu::ops::Operation;
use cpu::opcode::Opcode;
use cpu::pointer::Pointer;
use cpu::timer::Timer;
use memory::Memory;
use output::{font, graphics};

use {Address, Byte};
type Register = usize;

#[derive(Debug, PartialEq)]
pub struct Cpu<'a, G: 'a> where G: graphics::GraphicsOutput {
    pub exit: bool,
    pub beep: bool,
    pub draw: bool,
    pc: Pointer,
    sp: Pointer,
    i: Pointer,
    dt: Timer,
    st: Timer,
    v: [Byte; NUM_REGISTERS],
    memory: Memory,
    graphics: &'a G
}

impl<'a, G> Cpu<'a, G> where G: graphics::GraphicsOutput {
    pub fn new(rom: &Vec<Byte>, graphics: &'a G) -> Cpu<'a, G> {
        let mut memory = Memory::new();
        memory.load(&font::FONT_SET, FONT_RANGE);
        memory.load(&rom, ROM_RANGE);

        Cpu {
            exit: false,
            beep: true,
            draw: false,
            pc: Pointer::new(ROM_RANGE),
            sp: Pointer::new(STACK_RANGE),
            i: Pointer::new(FONT_RANGE.start..DISPLAY_RANGE.end),
            dt: Timer::new(60, 60),
            st: Timer::new(60, 60),
            v: [0x0; NUM_REGISTERS],
            memory,
            graphics
        }
    }

    pub fn step(&mut self) {
        self.draw = false;

        let opcode = self.fetch_opcode();
        let op = self.operation(&opcode);
        op(self, &opcode);

        self.update_timers();
    }

    pub fn fetch_opcode(&self) -> Opcode {
        let current = self.pc.current;
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

    fn skip(&mut self) {
        self.pc.move_forward();
    }

    fn read_i(&self) -> Address {
        self.i.current
    }

    fn load_i(&mut self, addr: Address) {
        self.i.set(addr);
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

    fn read_image_bytes(&self, n: usize) -> Vec<Byte> {
        let i = self.read_i();
        self.memory[i..i + n].to_vec()
    }

    fn load_image_byte(&mut self, from_addr: Address, to_byte: Byte) -> bool {
        let from_byte = self.memory[from_addr];
        let new_byte = from_byte ^ to_byte;
        if new_byte != from_byte {
            self.memory[from_addr] = new_byte;
            self.draw = true;
        }
        (from_byte & to_byte) > 0
    }

    fn load_image_bytes(&mut self, to_bytes: Vec<Byte>) -> bool {
        let i = self.read_i();
        to_bytes.iter().enumerate().any(|(b, to_byte)| {
            let from_addr = i + b;
            self.load_image_byte(from_addr, *to_byte)
        })
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
        let current = self.sp.current;
        let addr = (self.memory[current] as Address) << 8 | (self.memory[current + 1] as Address);
        self.sp.move_backward();
        addr
    }

    fn stack_push(&mut self) {
        self.sp.move_forward();
        let current = self.sp.current;
        let addr = self.pc.current;
        self.memory[current] = ((addr & 0xFF00) >> 8) as Byte;
        self.memory[current + 1] = (addr & 0x00FF) as Byte;
    }

    fn clear_display_data(&mut self) {
        for addr in DISPLAY_RANGE {
            self.memory[addr] = 0x0;
        }
    }

    pub fn display_data(&self) -> Vec<bool> {
        self.memory[DISPLAY_RANGE].iter()
            .fold(Vec::new(), |mut acc, byte| {
                let string = format!("{:08b}", byte);
                for c in string.chars() {
                    let b = c != '0';
                    acc.push(b);
                }
                acc
            })
    }
}

impl<'a, G> Operation for Cpu<'a, G> where G: graphics::GraphicsOutput {
    fn no_op(&mut self, _opcode: &Opcode) {
        self.pc.move_forward();
    }

    fn unknown(&mut self, opcode: &Opcode) {
        panic!("Unknown opcode {}", opcode);
    }

    fn clear_display(&mut self, _opcode: &Opcode) {
        self.clear_display_data();
        self.pc.move_forward();
        println!("\tCLS");
    }

    fn return_from_subroutine(&mut self, _opcode: &Opcode) {
        let addr = self.stack_pop();
        self.pc.set(addr);
        self.pc.move_forward();
        println!("\tRTN => {:x}", addr);
    }

    fn jump_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();

        if self.pc.current == addr {
            self.exit = true;
            return;
        }

        self.pc.set(addr);
        println!("\tJP {:x}", addr);
    }

    fn call_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.stack_push();
        self.pc.set(addr);
        println!("\tCALL {:x}", addr);
    }

    fn skip_equal_vx_byte(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let byte = opcode.kk();
        if vx == byte { self.skip(); }
        self.pc.move_forward();
        println!("\tSE vx: {:x}, byte: {:x}", vx, byte);
    }

    fn skip_not_equal_vx_byte(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let byte = opcode.kk();
        if vx != byte { self.skip(); }
        self.pc.move_forward();
        println!("\tSNE vx: {:x}, byte: {:x}", vx, byte);
    }

    fn skip_equal_vx_vy(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let vy = self.read_register(opcode.y());
        if vx == vy { self.skip(); }
        self.pc.move_forward();
        println!("\tSE vx: {:x}, vy: {:x}", vx, vy);
    }

    fn load_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        self.load_register(x, opcode.kk());
        self.pc.move_forward();
        println!("\tLD V{:x}, byte: {:x} => {:x}", x, opcode.kk(), self.read_register(x));
    }

    fn add_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let byte = opcode.kk();
        let vx = self.read_register(x);
        self.load_register(x, vx.wrapping_add(byte));
        self.pc.move_forward();
        println!("\tADD V{:x}: {:x}, byte: {:x} => {:x}", x, vx, byte, self.read_register(x));
    }

    fn load_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        self.load_register(x, vy);
        self.pc.move_forward();
        println!("\tLD V{:x}, V{:x}: {:x} => {:x}", x, y, vy, self.read_register(x));
    }

    fn or_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx | vy);
        self.pc.move_forward();
        println!("\tOR V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn and_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx & vy);
        self.pc.move_forward();
        println!("\tAND V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn xor_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx ^ vy);
        self.pc.move_forward();
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
        self.pc.move_forward();
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
        self.pc.move_forward();
        println!("\tSUB V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn shr_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        let bit = vy % 2;
        self.load_register(x, vy.wrapping_shr(1));
        self.load_flag(bit == 1);
        self.pc.move_forward();
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
        self.pc.move_forward();
        println!("\tSUBN V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn shl_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        let bit = vy >> 7;
        self.load_register(x, vy.wrapping_shl(1));
        self.load_flag(bit == 1);
        self.pc.move_forward();
        println!("\tSHL V{:x}, V{:x}: {:x} => ({:x}, {})", x, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn skip_not_equal_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        if vx != vy { self.skip(); }
        self.pc.move_forward();
        println!("\tSNE V{:x}: {:x}, V{:x}: {:x}", x, vx, y, vy);
    }

    fn load_i_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.load_i(addr);
        self.pc.move_forward();
        println!("\tLD I, {:x} => {:x}", addr, self.read_i());
    }

    fn jump_v0_addr(&mut self, opcode: &Opcode) {
        let v0 = self.read_register(0x0) as Address;
        let addr = opcode.nnn();
        self.pc.set(addr + v0);
        println!("\tJP V0: {:x}, {:x}", v0, addr);
    }

    fn rand_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let byte = opcode.kk();
        let random_byte: Byte = rand::thread_rng().gen_range(0x0, 0xFF);
        self.load_register(x, random_byte & byte);
        self.pc.move_forward();
        println!("\tRND V{:x} => {:x}", x, self.read_register(x));
    }

    fn draw_vx_vy_n(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x) as Address;
        let vy = self.read_register(y) as Address;
        let n = opcode.k();

        let sprite_bytes = self.read_image_bytes(n);
        let byte_x = (vx / graphics::SCREEN_WIDTH_SPRITES) % graphics::SCREEN_WIDTH_SPRITES;
        let pixel_x = vx % graphics::SPRITE_WIDTH;

        let mut collision = false;
        for sprite_y in 0..n {
            let byte_y = (vy + sprite_y) % graphics::SCREEN_HEIGHT;
            let offset_y = DISPLAY_RANGE.start + byte_y * graphics::SCREEN_WIDTH_SPRITES;
            let sprite_byte = sprite_bytes[sprite_y];
            match pixel_x {
                0 => {
                    let from_addr = offset_y + byte_x;
                    collision = collision | self.load_image_byte(from_addr, sprite_byte);
                },
                _ => {
                    let shift_r = pixel_x as u32;
                    let left = sprite_byte.wrapping_shr(shift_r) & 0xFFu8.wrapping_shr(shift_r);
                    let left_addr = offset_y + byte_x;

                    let shift_l = (graphics::SPRITE_WIDTH - pixel_x) as u32;
                    let right = sprite_byte.wrapping_shl(shift_l) & 0xFFu8.wrapping_shl(shift_l);
                    let right_x = (byte_x + 1) % graphics::SCREEN_WIDTH_SPRITES;
                    let right_addr = offset_y + right_x;

                    collision = collision
                        | self.load_image_byte(left_addr, left)
                        | self.load_image_byte(right_addr, right);
                }
            };
        }

        self.load_flag(collision);
        self.pc.move_forward();

        println!("\tDRW V{:x}: {:x}, V{:x}: {:x}, {:?}", x, vx, y, vy, sprite_bytes);
    }

    fn skip_key_pressed_vx(&mut self, _opcode: &Opcode) {}
    fn skip_key_not_pressed_vx(&mut self, _opcode: &Opcode) {}

    fn load_vx_dt(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let dt = self.read_delay_timer();
        self.load_register(x, dt);
        self.pc.move_forward();
        println!("\tLD V{:x}, DT: {:x} => {:x}", x, dt, self.read_register(x));
    }

    fn load_vx_key(&mut self, _opcode: &Opcode) {}

    fn load_dt_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        self.load_delay_timer(vx);
        self.pc.move_forward();
        println!("\tLD DT, V{:x}: {:x} => {:x}", x, vx, self.read_delay_timer());
    }

    fn load_st_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        self.load_sound_timer(vx);
        self.pc.move_forward();
        println!("\tLD ST, V{:x}: {:x} => {:x}", x, vx, self.read_sound_timer());
    }

    fn add_i_vx(&mut self, opcode: &Opcode) {
        let i = self.read_i();
        let x = opcode.x();
        let vx = self.read_register(x) as Address;
        self.load_i(i.wrapping_add(vx));
        self.pc.move_forward();
        println!("\tADD I: {:x}, V{:x}: {:x} => {:x}", i, x, vx, self.read_i());
    }

    fn load_i_vx_font(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x) as Address;
        self.load_i(vx * font::SPRITE_HEIGHT);
        self.pc.move_forward();
        println!("\tLD I, FONT V{:x}: {:x} => {:x}", x, vx, self.read_i());
    }

    fn load_bcd_vx(&mut self, opcode: &Opcode) {
        let i = self.read_i();
        let x = opcode.x();
        let vx = self.read_register(x);
        self.memory[i + 0] = vx / 100;
        self.memory[i + 1] = vx % 100 / 10;
        self.memory[i + 2] = vx % 10;
        self.pc.move_forward();
        println!("\tLD BCD V{:x}: {:x} ({}) => {:?}", x, vx, vx, self.read_image_bytes(3));
    }

    fn load_through_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let i = self.read_i();
        let bytes = (0..x + 1).map(|r| self.read_register(r)).collect();
        self.load_image_bytes(bytes);
        self.load_i(i + x + 1);
        self.pc.move_forward();

        let register_bytes: Vec<Byte> = (0..x + 1).map(|r| self.read_register(r)).collect();
        println!("\tLD [I], V{:x} [{:?}] => {:?}", x, register_bytes, self.read_image_bytes(x + 1));
    }

    fn read_through_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let i = self.read_i();
        let bytes = self.read_image_bytes(x + 1);
        for (r, byte) in bytes.iter().enumerate() {
            self.load_register(r, *byte);
        }
        self.load_i(i + x + 1);
        self.pc.move_forward();

        let register_bytes: Vec<Byte> = (0..x + 1).map(|r| self.read_register(r)).collect();
        println!("\tRD V{:x} [{:?}], [I] => {:?}", x, self.read_image_bytes(x + 1), register_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_loads_font_to_memory() {
        let rom = Vec::new();
        let cpu = Cpu::new(&rom);

        let font_set = font::FONT_SET.to_vec();
        let range = FONT_RANGE.start..FONT_RANGE.start + font::FONT_SET.len();
        let mem = cpu.memory[range].to_vec();
        assert_eq!(font_set, mem);
    }

    #[test]
    fn new_loads_rom_to_memory() {
        let rom = vec![0x00, 0x01, 0x02, 0x03];
        let cpu = Cpu::new(&rom);
        
        let range = ROM_RANGE.start..ROM_RANGE.start + rom.len();
        let mem = cpu.memory[range].to_vec();
        assert_eq!(rom, mem);
    }

    #[test]
    fn fetch_opcode_fetches_two_current_bytes() {
        let rom = vec![0xAB, 0xCD, 0xEF, 0xFF];
        let cpu = Cpu::new(&rom);

        assert_eq!(Opcode::new(0xABCD), cpu.fetch_opcode());        
    }

    #[test]
    fn operation_0000_no_op() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x0000);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;
        let memory = cpu.memory.clone();

        op(&mut cpu, &opcode);
        assert_eq!(memory, cpu.memory);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_00e0_clear_display() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x00E0);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        for i in DISPLAY_RANGE {
            cpu.memory[i] = 0xFF;
        }

        op(&mut cpu, &opcode);
        assert!(cpu.display_data().iter().all(|data| *data == false));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_00ee_return_from_subroutine() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x00EE);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.stack_push();
        cpu.pc.move_forward();
        cpu.pc.move_forward();
        cpu.pc.move_forward();

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_1nnn_jump_addr() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x1404);
        let op = cpu.operation(&opcode);

        op(&mut cpu, &opcode);
        assert_eq!(0x404, cpu.pc.current);
    }

    #[test]
    fn operation_2nnn_call_subroutine() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x2404);
        let op = cpu.operation(&opcode);

        op(&mut cpu, &opcode);
        assert_eq!(0x404, cpu.pc.current);
        assert_eq!(0x02, cpu.memory[cpu.sp.current]);
        assert_eq!(0x00, cpu.memory[cpu.sp.current + 1]);
    }

    #[test]
    fn operation_3xkk_skip_equal_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x3123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc.current);
    }

    #[test]
    fn operation_3xkk_skip_equal_vx_byte_no_skip() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x3122);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;
        
        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_4xkk_skip_not_equal_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x4123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0x22;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc.current);
    }

    #[test]
    fn operation_4xkk_skip_not_equal_vx_byte_no_skip() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x4123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_5xy0_skip_equal_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x5010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = cpu.v[0x0];

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc.current);
    }

    #[test]
    fn operation_5xy0_skip_equal_vx_vy_no_skip() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x5010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = 0x22;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_6xkk_load_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x6123);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        op(&mut cpu, &opcode);
        assert_eq!(0x23, cpu.read_register(0x1));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_7xkk_add_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x71FF);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x1));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_7xkk_add_vx_byte_wrap() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x71FF);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x1));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy0_load_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0x23;

        op(&mut cpu, &opcode);
        assert_eq!(0x23, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_00() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_01() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_10() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy1_or_vx_vy_11() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8011);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_00() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_01() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_10() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy2_and_vx_vy_11() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_00() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_01() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x0;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_10() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x0;

        op(&mut cpu, &opcode);
        assert_eq!(0x1, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy3_xor_vx_vy_11() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8013);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x1;
        cpu.v[0x1] = 0x1;

        op(&mut cpu, &opcode);
        assert_eq!(0x0, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy4_add_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8014);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x03, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy4_add_vx_vy_carry() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8014);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0xFF;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy5_sub_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8015);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x02;
        cpu.v[0x1] = 0x01;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy5_sub_vx_vy_carry() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8015);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy6_shr_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8016);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0b11111110;

        op(&mut cpu, &opcode);
        assert_eq!(0b01111111, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy6_shr_vx_vy_sig_bit() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8016);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0b11111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b01111111, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy7_subn_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8017);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xy7_subn_vx_vy_carry() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x8017);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x02;
        cpu.v[0x1] = 0x01;

        op(&mut cpu, &opcode);
        assert_eq!(0xFF, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xye_shl_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x801E);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0b01111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b11111110, cpu.read_register(0x0));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_8xye_shl_vx_vy_sig_bit() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x801E);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x1] = 0b11111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b11111110, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));   
        assert_eq!(pc + 2, cpu.pc.current);     
    }

    #[test]
    fn operation_9xy0_skip_not_equal_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x9010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = 0x22;

        op(&mut cpu, &opcode);
        assert_eq!(pc + 4, cpu.pc.current);
    }

    #[test]
    fn operation_9xy0_skip_not_equal_vx_vy_no_skip() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0x9010);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x23;
        cpu.v[0x1] = cpu.v[0x0];

        op(&mut cpu, &opcode);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_annn_load_i_addr() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xA456);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        op(&mut cpu, &opcode);
        assert_eq!(0x456, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_bnnn_jump_v0_addr() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xB300);
        let op = cpu.operation(&opcode);

        cpu.v[0x0] = 0x08;

        op(&mut cpu, &opcode);
        assert_eq!(0x308, cpu.pc.current);
    }

    #[test]
    fn operation_cxkk_rand_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xC00F);
        let op = cpu.operation(&opcode);

        for _ in 0..1000 {
            let pc = cpu.pc.current;

            op(&mut cpu, &opcode);
            assert!(cpu.read_register(0x0) <= 0xF);
            assert_eq!(pc + 2, cpu.pc.current);
        }
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 8;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b10011001;
        cpu.memory[i + 1] = 0b11111111;

        op(&mut cpu, &opcode);
        let addr_0 = DISPLAY_RANGE.start + 5 * 8 + 1;
        let addr_1 = DISPLAY_RANGE.start + 6 * 8 + 1;
        assert_eq!(0b10011001, cpu.memory[addr_0]);
        assert_eq!(0b11111111, cpu.memory[addr_1]);
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_collision() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 8;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b00000001;
        cpu.memory[i + 1] = 0b11111111;

        let addr_0 = DISPLAY_RANGE.start + 5 * 8 + 1;
        let addr_1 = DISPLAY_RANGE.start + 6 * 8 + 1;
        cpu.memory[addr_0] = 0b10000001;
        cpu.memory[addr_1] = 0b11111111;

        op(&mut cpu, &opcode);
        assert_eq!(0b10000000, cpu.memory[addr_0]);
        assert_eq!(0b00000000, cpu.memory[addr_1]);
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw)
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vx_offset() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 2;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b10011001;
        cpu.memory[i + 1] = 0b11111111;

        op(&mut cpu, &opcode);
        let addr_0 = DISPLAY_RANGE.start + 5 * 8;
        let addr_1 = DISPLAY_RANGE.start + 6 * 8;
        assert_eq!((0b00100110, 0b01000000), (cpu.memory[addr_0], cpu.memory[addr_0 + 1]));
        assert_eq!((0b00111111, 0b11000000), (cpu.memory[addr_1], cpu.memory[addr_1 + 1]));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vx_offset_collision() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 2;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b10011001;
        cpu.memory[i + 1] = 0b11111111;

        let addr_0 = DISPLAY_RANGE.start + 5 * 8;
        let addr_1 = DISPLAY_RANGE.start + 6 * 8;
        cpu.memory[addr_0 + 0] = 0b00100110;
        cpu.memory[addr_0 + 1] = 0b01000000;
        cpu.memory[addr_1 + 0] = 0b00111111;
        cpu.memory[addr_1 + 1] = 0b11000000;

        op(&mut cpu, &opcode);
        assert_eq!((0b0, 0b0), (cpu.memory[addr_0], cpu.memory[addr_0 + 1]));
        assert_eq!((0b0, 0b0), (cpu.memory[addr_1], cpu.memory[addr_1 + 1]));
        assert_eq!(0b1, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vx_wrap() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 64;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b10011001;
        cpu.memory[i + 1] = 0b11111111;

        op(&mut cpu, &opcode);
        let addr_0 = DISPLAY_RANGE.start + 5 * 8;
        let addr_1 = DISPLAY_RANGE.start + 6 * 8;
        assert_eq!(0b10011001, cpu.memory[addr_0]);
        assert_eq!(0b11111111, cpu.memory[addr_1]);
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vx_wrap_offset() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 66;
        cpu.v[1] = 5;
        cpu.memory[i + 0] = 0b10011001;
        cpu.memory[i + 1] = 0b11111111;

        op(&mut cpu, &opcode);
        let addr_0 = DISPLAY_RANGE.start + 5 * 8;
        let addr_1 = DISPLAY_RANGE.start + 6 * 8;
        assert_eq!((0b00100110, 0b01000000), (cpu.memory[addr_0], cpu.memory[addr_0 + 1]));
        assert_eq!((0b00111111, 0b11000000), (cpu.memory[addr_1], cpu.memory[addr_1 + 1]));
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_with_vy_wrap() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        let i = cpu.read_i();
        cpu.v[0] = 8;
        cpu.v[1] = 31;
        cpu.memory[i + 0] = 0b10011001;
        cpu.memory[i + 1] = 0b11111111;

        op(&mut cpu, &opcode);
        let addr_0 = DISPLAY_RANGE.start + 31 * 8 + 1;
        let addr_1 = DISPLAY_RANGE.start + 0 * 8 + 1;
        assert_eq!(0b10011001, cpu.memory[addr_0]);
        assert_eq!(0b11111111, cpu.memory[addr_1]);
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(cpu.draw);
    }

    #[test]
    fn operation_dxyn_draw_vx_vy_n_no_change() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xD012);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.load_i(ROM_RANGE.start);
        cpu.v[0] = 0;
        cpu.v[1] = 0;

        op(&mut cpu, &opcode);
        let addr_0 = DISPLAY_RANGE.start;
        let addr_1 = DISPLAY_RANGE.start + 1 * 8;
        assert_eq!(0b00000000, cpu.memory[addr_0]);
        assert_eq!(0b00000000, cpu.memory[addr_1]);
        assert_eq!(0b0, cpu.read_register(0xF));
        assert_eq!(pc + 2, cpu.pc.current);
        assert!(!cpu.draw);
    }

    #[test]
    fn operation_fx07_load_vx_dt() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF007);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        let time = 0x20;
        cpu.dt.set(time);

        op(&mut cpu, &opcode);
        assert_eq!(time, cpu.read_register(0x0));
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx15_load_dt_vx() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF015);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        let time = 0x20;
        cpu.v[0x0] = time;

        op(&mut cpu, &opcode);
        assert_eq!(time, cpu.read_delay_timer());
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx18_load_st_vx() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF018);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        let time = 0x20;
        cpu.v[0x0] = time;

        op(&mut cpu, &opcode);
        assert_eq!(time, cpu.read_sound_timer());
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx1e_add_i_vx() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF01E);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;
        let i = cpu.read_i();

        cpu.v[0x0] = 0x08;

        op(&mut cpu, &opcode);
        assert_eq!(i + 0x08, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx29_load_i_vx_font() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF029);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

        cpu.v[0x0] = 0x02;

        op(&mut cpu, &opcode);
        assert_eq!(FONT_RANGE.start + 10, cpu.read_i());
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx33_load_bcd_vx() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF033);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;
        let i = cpu.read_i();

        cpu.v[0x0] = 0xFE;

        op(&mut cpu, &opcode);
        assert_eq!(0x02, cpu.memory[i + 0]);
        assert_eq!(0x05, cpu.memory[i + 1]);
        assert_eq!(0x04, cpu.memory[i + 2]);
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx55_load_through_vx() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF555);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

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
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_fx65_read_through_vx() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);
        let opcode = Opcode::new(0xF565);
        let op = cpu.operation(&opcode);
        let pc = cpu.pc.current;

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
        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn beep_while_sound_timer_active() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        cpu.update_timers();

        assert!(cpu.st.active());
        assert!(cpu.beep);

        cpu.st.set(0);
        cpu.update_timers();

        assert!(!cpu.st.active());
        assert!(!cpu.beep);
    }
}
