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
use std::fmt::{self, Display};

use cpu::ops::Operation;
use cpu::opcode::Opcode;
use cpu::pointer::Pointer;
use cpu::timer::Timer;
use memory::Memory;
use output::{font, graphics};

use {Address, Byte};
type Register = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct Cpu {
    pub exit: bool,
    pc: Pointer,
    sp: Pointer,
    i: Pointer,
    dt: Timer,
    st: Timer,
    v: [Byte; NUM_REGISTERS],
    memory: Memory
}

impl Cpu {
    pub fn new(rom: &Vec<Byte>) -> Cpu {
        let mut memory = Memory::new();
        memory.load(&font::FONT_SET, FONT_RANGE);
        memory.load(&rom, ROM_RANGE);

        Cpu {
            exit: false,
            pc: Pointer::new(ROM_RANGE),
            sp: Pointer::new(STACK_RANGE),
            i: Pointer::new(FONT_RANGE.start..ROM_RANGE.end),
            dt: Timer::new(60),
            st: Timer::new(60),
            v: [0x0; NUM_REGISTERS],
            memory: memory
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_opcode();
        let op = self.operation(&opcode);
        println!("[#{:x}] {}", self.pc.current, opcode);
        op(self, &opcode);

        self.pc.move_forward();

        self.dt.tick();
        self.st.tick();
    }

    pub fn fetch_opcode(&self) -> Opcode {
        let current = self.pc.current;
        let bytes = (self.memory[current], self.memory[current + 1]);
        Opcode::from_bytes(bytes)
    }

    pub fn operation(&mut self, opcode: &Opcode) -> fn(&mut Cpu, &Opcode) {
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

    fn jump(&mut self, addr: Address) {
        self.pc.set(addr);
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

    fn load_image_byte(&mut self, x: Address, y: Address, to_byte: Byte) -> bool {
        let from_addr = DISPLAY_RANGE.start + y * graphics::SCREEN_WIDTH_SPRITES + x;
        let from_byte = self.memory[from_addr];
        self.memory[from_addr] = from_byte ^ to_byte;
        (from_byte & to_byte) > 0
    }

    fn load_image_bytes(&mut self, to_bytes: Vec<Byte>) -> bool {
        let i = self.read_i();
        to_bytes.iter().any(|to_byte| {
            self.load_image_byte(i, 0, *to_byte)
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

impl Operation for Cpu {
    fn no_op(&mut self, _opcode: &Opcode) {}

    fn unknown(&mut self, opcode: &Opcode) {
        panic!("Unknown opcode {}", opcode);
    }

    fn clear_display(&mut self, _opcode: &Opcode) {
        self.clear_display_data();
        println!("\tCLS");
    }

    fn return_from_subroutine(&mut self, _opcode: &Opcode) {
        let addr = self.stack_pop();
        self.jump(addr);
        println!("\tRTN => {:x}", addr);
    }

    fn jump_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.jump(addr);
        println!("\tJP {:x}", addr);
    }

    fn call_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.stack_push();
        self.jump(addr);
        println!("\tCALL {:x}", addr);
    }

    fn skip_equal_vx_byte(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let byte = opcode.kk();
        if vx == byte { self.skip(); }
        println!("\tSE vx: {:x}, byte: {:x}", vx, byte);
    }

    fn skip_not_equal_vx_byte(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let byte = opcode.kk();
        if vx != byte { self.skip(); }
        println!("\tSNE vx: {:x}, byte: {:x}", vx, byte);
    }

    fn skip_equal_vx_vy(&mut self, opcode: &Opcode) {
        let vx = self.read_register(opcode.x());
        let vy = self.read_register(opcode.y());
        if vx == vy { self.skip(); }
        println!("\tSE vx: {:x}, vy: {:x}", vx, vy);
    }

    fn load_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        self.load_register(x, opcode.kk());
        println!("\tLD V{:x}, byte: {:x} => {:x}", x, opcode.kk(), self.read_register(x));
    }

    fn add_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let byte = opcode.kk();
        let vx = self.read_register(x);
        self.load_register(x, vx.wrapping_add(byte));
        println!("\tADD V{:x}: {:x}, byte: {:x} => {:x}", x, vx, byte, self.read_register(x));
    }

    fn load_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        self.load_register(x, vy);
        println!("\tLD V{:x}, V{:x}: {:x} => {:x}", x, y, vy, self.read_register(x));
    }

    fn or_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx | vy);
        println!("\tOR V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn and_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx & vy);
        println!("\tAND V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, self.read_register(x));
    }

    fn xor_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        self.load_register(x, vx ^ vy);
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
        println!("\tSUB V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn shr_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        let (result, overflow) = vy.overflowing_shr(1);
        self.load_register(x, result);
        self.load_flag(overflow);
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
        println!("\tSUBN V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn shl_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vy = self.read_register(y);
        let (result, overflow) = vy.overflowing_shl(1);
        self.load_register(x, result);
        self.load_flag(overflow);
        println!("\tSHL V{:x}, V{:x}: {:x} => ({:x}, {})", x, y, vy, self.read_register(x), self.read_register(0xF));
    }

    fn skip_not_equal_vx_vy(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let vx = self.read_register(x);
        let vy = self.read_register(y);
        if vx != vy { self.skip(); }
        println!("\tSNE V{:x}: {:x}, V{:x}: {:x}", x, vx, y, vy);
    }

    fn load_i_addr(&mut self, opcode: &Opcode) {
        let addr = opcode.nnn();
        self.load_i(addr);
        println!("\tLD I, {:x} => {:x}", addr, self.read_i());
    }

    fn jump_v0_addr(&mut self, opcode: &Opcode) {
        let v0 = self.read_register(0x0) as Address;
        let addr = opcode.nnn();
        self.jump(addr + v0);
        println!("\tJP V0: {:x}, {:x}", v0, addr);
    }

    fn rand_vx_byte(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let byte = opcode.kk();
        let random_byte: Byte = rand::thread_rng().gen_range(0x0, 0xFF);
        self.load_register(x, random_byte & byte);
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

        for sprite_y in 0..n {
            let byte_y = (vy + sprite_y) % graphics::SCREEN_HEIGHT;
            let sprite_byte = sprite_bytes[sprite_y];
            let mut collision = match pixel_x {
                0 => self.load_image_byte(byte_x, byte_y, sprite_byte),
                _ => {
                    let shift_r = pixel_x as u32;
                    let left = sprite_byte.wrapping_shr(shift_r);

                    let shift_l = (graphics::SPRITE_WIDTH - pixel_x) as u32;
                    let right = sprite_byte.wrapping_shl(shift_l);
                    let right_x = (byte_x + 1) % graphics::SCREEN_WIDTH_SPRITES;

                    self.load_image_byte(byte_x, byte_y, left) &
                        self.load_image_byte(right_x, byte_y, right)
                }
            };

            self.load_flag(collision);
        }

        println!("\tDRW V{:x}: {:x}, V{:x}: {:x}, {:?}", x, vx, y, vy, sprite_bytes);
        print!("   ");
        for i in 0..graphics::SCREEN_WIDTH_SPRITES {
            print!("{}               ", i);
        }
        println!("{}", self);
    }

    fn skip_key_pressed_vx(&mut self, _opcode: &Opcode) {}
    fn skip_key_not_pressed_vx(&mut self, _opcode: &Opcode) {}

    fn load_vx_dt(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let dt = self.read_delay_timer();
        self.load_register(x, dt);
        println!("\tLD V{:x}, DT: {:x} => {:x}", x, dt, self.read_register(x));
    }

    fn load_vx_key(&mut self, _opcode: &Opcode) {}

    fn load_dt_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        self.load_delay_timer(vx);
        println!("\tLD DT, V{:x}: {:x} => {:x}", x, vx, self.read_delay_timer());
    }

    fn load_st_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        self.load_sound_timer(vx);
        println!("\tLD ST, V{:x}: {:x} => {:x}", x, vx, self.read_sound_timer());
    }

    fn add_i_vx(&mut self, opcode: &Opcode) {
        let i = self.read_i();
        let x = opcode.x();
        let vx = self.read_register(x) as Address;
        self.load_i(i.wrapping_add(vx));
        println!("\tADD I: {:x}, V{:x}: {:x} => {:x}", i, x, vx, self.read_i());
    }

    fn load_i_vx_font(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x) as Address;
        self.load_i(vx * font::SPRITE_HEIGHT);
        println!("\tLD I, FONT V{:x}: {:x} => {:x}", x, vx, self.read_i());
    }

    fn load_bcd_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let vx = self.read_register(x);
        let bytes = vec![
            vx / 100,
            vx % 100 / 10,
            vx % 10
        ];
        self.load_image_bytes(bytes);
        println!("\tLD BCD V{:x}: {:x} ({}) => {:?}", x, vx, vx, self.read_image_bytes(3));
    }

    fn load_through_vx(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let i = self.read_i();
        let bytes = (0..x + 1).map(|r| self.read_register(r)).collect();
        self.load_image_bytes(bytes);
        self.load_i(i + x + 1);

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

        let register_bytes: Vec<Byte> = (0..x + 1).map(|r| self.read_register(r)).collect();
        println!("\tRD V{:x} [{:?}], [I] => {:?}", x, self.read_image_bytes(x + 1), register_bytes);
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lines = self.display_data().iter().enumerate()
            .fold(String::new(), |mut acc, (i, bit)| {
                if (i % graphics::SCREEN_WIDTH) == 0 {
                    acc.push_str(&format!("\n{:02} ", i / 64));
                }

                let c = match *bit {
                    true => "  ",
                    false => "▓▓︎"
                };

                acc.push_str(c);
                acc
            });
        write!(f, "{}", lines)
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

        let clone = cpu.clone();

        let opcode = Opcode::new(0x0000);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(clone, cpu);
    }

    #[test]
    fn operation_00e0_clear_display() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        for i in DISPLAY_RANGE {
            cpu.memory[i] = 0xFF;
        }

        let opcode = Opcode::new(0x00E0);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert!(cpu.display_data().iter().all(|data| *data == false));
    }

    #[test]
    fn operation_00ee_return_from_subroutine() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        let pc = cpu.pc.current;
        cpu.stack_push();
        cpu.pc.move_forward();
        cpu.pc.move_forward();
        cpu.pc.move_forward();

        let opcode = Opcode::new(0x00EE);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc, cpu.pc.current);
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

        let pc = cpu.pc.current;
        cpu.v[0x1] = 0x23;

        let opcode = Opcode::new(0x3122);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc, cpu.pc.current);

        let opcode = Opcode::new(0x3123);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_4xkk_skip_not_equal_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        let pc = cpu.pc.current;
        cpu.v[0x1] = 0x23;

        let opcode = Opcode::new(0x4123);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc, cpu.pc.current);

        let opcode = Opcode::new(0x4122);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_5xy0_skip_not_equal_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        let pc = cpu.pc.current;
        cpu.v[0x0] = 0x22;
        cpu.v[0x1] = 0x23;

        let opcode = Opcode::new(0x5010);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc, cpu.pc.current);

        cpu.v[0x1] = cpu.v[0x0];

        let opcode = Opcode::new(0x5010);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(pc + 2, cpu.pc.current);
    }

    #[test]
    fn operation_6xkk_load_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        let opcode = Opcode::new(0x6123);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(0x23, cpu.read_register(0x1));
    }

    #[test]
    fn operation_7xkk_add_vx_byte() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        let opcode = Opcode::new(0x71FF);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(0xFF, cpu.read_register(0x1));

        let opcode = Opcode::new(0x7102);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(0x01, cpu.read_register(0x1));
    }

    #[test]
    fn operation_8xy0_load_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        cpu.v[0x1] = 0x23;

        let opcode = Opcode::new(0x8010);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(0x23, cpu.read_register(0x0));
    }

    #[test]
    fn operation_8xy4_add_vx_vy() {
        let rom = Vec::new();
        let mut cpu = Cpu::new(&rom);

        cpu.v[0x0] = 0xFF;
        cpu.v[0x1] = 0x02;

        let opcode = Opcode::new(0x8014);
        let op = cpu.operation(&opcode);
        op(&mut cpu, &opcode);

        assert_eq!(0x01, cpu.read_register(0x0));
        assert_eq!(0b1, cpu.read_register(0xF));
    }
}
