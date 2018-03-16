const NUM_REGISTERS: usize = 16;

const FONT_RANGE: Range<Address> = 0x0..0x200;
const ROM_RANGE: Range<Address> = 0x200..0xEA0;
const STACK_RANGE: Range<Address> = 0xEA0..0xF00;
const DISPLAY_RANGE: Range<Address> = 0xF00..Memory::MAX_SIZE;

use std::ops::Range;
use std::fmt::{self, Display};

use memory::Memory;
use opcode::Opcode;
use ops;
use output::{font, graphics};
use pointer::Pointer;
use timer::Timer;

use {Address, Byte};
type Register = usize;

#[derive(Debug)]
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

    pub fn fetch_opcode(&mut self) -> Opcode {
        let current = self.pc.current;
        let bytes = (self.memory[current], self.memory[current + 1]);
        Opcode::from_bytes(bytes)
    }

    pub fn operation(&mut self, opcode: &Opcode) -> fn(&mut Cpu, &Opcode) {
        match opcode.first_hex_digit() {
            0x0 => {
                match opcode.kk() {
                    0x00 => ops::no_op,
                    0xE0 => ops::clear_display,
                    0xEE => ops::return_from_subroutine,
                    _ => ops::unknown
                }
            },
            0x1 => ops::jump_addr,
            0x2 => ops::call_addr,
            0x3 => ops::skip_equal_vx_byte,
            0x4 => ops::skip_not_equal_vx_byte,
            0x5 => ops::skip_equal_vx_vy,
            0x6 => ops::load_vx_byte,
            0x7 => ops::add_vx_byte,
            0x8 => {
                match opcode.k() {
                    0x0 => ops::load_vx_vy,
                    0x1 => ops::or_vx_vy,
                    0x2 => ops::and_vx_vy,
                    0x3 => ops::xor_vx_vy,
                    0x4 => ops::add_vx_vy,
                    0x5 => ops::sub_vx_vy,
                    0x6 => ops::shr_vx_vy,
                    0x7 => ops::subn_vx_vy,
                    0xE => ops::shl_vx_vy,
                    _ => ops::unknown
                }
            },
            0x9 => ops::skip_not_equal_vx_vy,
            0xA => ops::load_i_addr,
            0xB => ops::jump_v0_addr,
            0xC => ops::rand_vx_byte,
            0xD => ops::draw_vx_vy_n,
            0xE => {
                match opcode.kk() {
                    0x9E => ops::skip_key_pressed_vx,
                    0xA1 => ops::skip_key_not_pressed_vx,
                    _ => ops::unknown
                }
            },
            0xF => {
                match opcode.kk() {
                    0x07 => ops::load_vx_dt,
                    0x0A => ops::load_vx_key,
                    0x15 => ops::load_dt_vx,
                    0x18 => ops::load_st_vx,
                    0x1E => ops::add_i_vx,
                    0x29 => ops::load_i_vx_font,
                    0x33 => ops::load_bcd_vx,
                    0x55 => ops::load_through_vx,
                    0x65 => ops::read_through_vx,
                    _ => ops::unknown
                }
            },
            _ => ops::unknown
        }
    }

    pub fn jump(&mut self, addr: Address) {
        self.pc.set(addr);
    }

    pub fn skip(&mut self) {
        self.pc.move_forward();
    }

    pub fn read_i(&self) -> Address {
        self.i.current
    }

    pub fn load_i(&mut self, addr: Address) {
        self.i.set(addr);
    }

    pub fn load_register(&mut self, register: Register, val: Byte) {
        self.v[register] = val;
    }

    pub fn read_register(&self, register: Register) -> Byte {
        self.v[register]
    }

    pub fn load_flag(&mut self, b: bool) {
        if b {
            self.load_register(0xF, 0b0);    
        } else {
            self.load_register(0xF, 0b1);
        }
    }

    pub fn read_image_bytes(&self, n: usize) -> Vec<Byte> {
        let i = self.read_i();
        self.memory[i..i + n].to_vec()
    }

    pub fn load_image_byte(&mut self, x: Address, y: Address, to_byte: Byte) -> bool {
        let from_addr = DISPLAY_RANGE.start + y * graphics::SCREEN_WIDTH_SPRITES + x;
        let from_byte = self.memory[from_addr];
        self.memory[from_addr] = from_byte ^ to_byte;
        (from_byte & to_byte) > 0
    }

    pub fn load_image_bytes(&mut self, to_bytes: Vec<Byte>) -> bool {
        let i = self.read_i();
        to_bytes.iter().any(|to_byte| {
            self.load_image_byte(i, 0, *to_byte)
        })
    }

    pub fn read_delay_timer(&self) -> Byte {
        self.dt.current
    }

    pub fn load_delay_timer(&mut self, val: Byte) {
        self.dt.set(val);
    }

    pub fn read_sound_timer(&mut self) -> Byte {
        self.st.current
    }

    pub fn load_sound_timer(&mut self, val: Byte) {
        self.st.set(val);
    }

    pub fn stack_pop(&mut self) -> Address {
        let current = self.sp.current;
        let addr = (self.memory[current] as Address) << 8 | (self.memory[current + 1] as Address);
        self.sp.move_backward();
        addr
    }

    pub fn stack_push(&mut self) {
        self.sp.move_forward();
        let current = self.sp.current;
        let addr = self.pc.current;
        self.memory[current] = ((addr & 0xFF00) >> 8) as Byte;
        self.memory[current + 1] = (addr & 0x00FF) as Byte;
    }

    pub fn clear_display(&mut self) {
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

impl Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lines = self.display_data().iter().enumerate()
            .fold(String::new(), |mut acc, (i, bit)| {
                if (i % graphics::SCREEN_WIDTH) == 0 {
                    acc.push_str(&format!("\n{:02} ", i / 64));
                }

                let c = match *bit {
                    true => "⬜️",
                    false => "⬛️"
                };

                acc.push_str(c);
                acc
            });
        write!(f, "{}", lines)
    }
}
