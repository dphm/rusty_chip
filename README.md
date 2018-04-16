# Rusty CHIP

CHIP-8 Interpreter in Rust

![CHIP-8 Logo in ASCII](/chip8.png)

## CHIP-8

CHIP-8 is an interpreted programming language run on a CHIP-8 virtual machine.

### CHIP-8 virtual machine

#### Memory

There are 4096 memory locations, each 1 byte in size, addressed from `0x000` to `0xFFF`.

```text
      FONT  0x000 - 0x200
       ROM  0x200 - 0xFA0
CALL STACK  0xFA0 - 0xFFF
--------------------------
     TOTAL  0x000 - 0xFFF
```

#### Registers

##### Data registers

There are 16 data registers, each 1 byte in size, addressed from `0x0` to `0xF`.

The `0xF` register is used as a flag for some operations, and should not be written by other programs.

##### Address registers

There is 1 address register, 2 bytes in size, called `I`.

#### Call stack

The call stack stores return addresses when subroutines are called.

Modern interpreters support at least 16 levels of nesting, requiring a call stack size of 32 bytes.

#### Timers

Timers count down to zero at a rate of 60 Hz.

##### Delay

Used for timing game events

##### Sound

Beeps when value is non-zero

#### Input

CHIP-8 was used with a hexidecimal keypad:

```text
,---,---,---,---,
| 1 | 2 | 3 | C |
|---|---|---|---|
| 4 | 5 | 6 | D |
|---|---|---|---|
| 7 | 8 | 9 | E |
|---|---|---|---|
| A | 0 | B | F |
'---'---'---'---'
```

Rusty CHIP instead uses `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, and hexidecimal characters (`0` - `F`).

#### Graphics

```text
,---------------------,
|(0, 0)   ...  (63, 0)|
| .                 . |
| .                 . |
| .                 . |
|(0, 31)  ... (63, 31)|
'---------------------'
```

The monochrome display is 64 pixels wide by 32 pixels high. A sprite is 8 pixels wide by 1 - 15 pixels high. Sprites are drawn to the screen using `XOR`. The `0xF` register flag is set to `1` if any `true` pixels become `false`, indicating a collision.

#### Sound

Rusty CHIP's beep tone is a 1000 Hz sine wave because I don't find it as annoying as the other frequencies I tried.

### CHIP-8 instruction set

| Opcode | Instruction |
| ------ | ----------- |
| 0NNN | Execute machine language subroutine at address NNN |
| 00E0 | Clear the screen |
| 00EE | Return from a subroutine |
| 1NNN | Jump to address NNN |
| 2NNN | Execute subroutine starting at address NNN |
| 3XNN | Skip the following instruction if the value of register VX equals NN |
| 4XNN | Skip the following instruction if the value of register VX is not equal to NN |
| 5XY0 | Skip the following instruction if the value of register VX is equal to the value of register VY |
| 6XNN | Store number NN in register VX |
| 7XNN | Add the value NN to register VX |
| 8XY0 | Store the value of register VY in register VX |
| 8XY1 | Set VX to VX OR VY |
| 8XY2 | Set VX to VX AND VY |
| 8XY3 | Set VX to VX XOR VY |
| 8XY4 | Add the value of register VY to register VX, Set VF to 01 if a carry occurs, Set VF to 00 if a carry does not occur |
| 8XY5 | Subtract the value of register VY from register VX, Set VF to 00 if a borrow occurs, Set VF to 01 if a borrow does not occur |
| 8XY6 | Store the value of register VY shifted right one bit in register VX, Set register VF to the least significant bit prior to the shift |
| 8XY7 | Set register VX to the value of VY minus VX, Set VF to 00 if a borrow occurs, Set VF to 01 if a borrow does not occur |
| 8XYE | Store the value of register VY shifted left one bit in register VX, Set register VF to the most significant bit prior to the shift |
| 9XY0 | Skip the following instruction if the value of register VX is not equal to the value of register VY |
| ANNN | Store memory address NNN in register I |
| BNNN | Jump to address NNN + V0 |
| CXNN | Set VX to a random number with a mask of NN |
| DXYN | Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I, Set VF to 01 if any set pixels are changed to unset, and 00 otherwise |
| EX9E | Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed |
| EXA1 | Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed |
| FX07 | Store the current value of the delay timer in register VX |
| FX0A | Wait for a keypress and store the result in register VX |
| FX15 | Set the delay timer to the value of register VX |
| FX18 | Set the sound timer to the value of register VX |
| FX1E | Add the value stored in register VX to register I |
| FX29 | Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX |
| FX33 | Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I+1, and I+2 |
| FX55 | Store the values of registers V0 to VX inclusive in memory starting at address I, I is set to I + X + 1 after operation |
| FX65 | Fill registers V0 to VX inclusive with the values stored in memory starting at address I, I is set to I + X + 1 after operation |

*Source: [Mastering CHIP-8 by Matthew Mikolay](http://mattmik.com/files/chip8/mastering/chip8.html)*

## Resources

### CHIP-8

- [Mastering CHIP-8 by Matthew Mikolay](http://mattmik.com/files/chip8/mastering/chip8.html)
- [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [CHIP-8 Wikipedia Page](https://en.wikipedia.org/wiki/CHIP-8)

### WebAssembly

- [WebAssembly](http://webassembly.org/)
- [WebAssembly on MDN](https://developer.mozilla.org/en-US/docs/WebAssembly)

#### Rust + WebAssembly

- [Rust + WebAssembly](https://rust-lang-nursery.github.io/rust-wasm/)
- [Hello, Rust! Demos](https://www.hellorust.com/demos/)

## Acknowledgements

Thank you to the following people who discussed, reviewed, or paired on this project with me!

* [alexcoco](https://github.com/alexcoco)
* [cfinucane](https://github.com/cfinucane)
* [jedahan](https://github.com/jedahan)
* [kamalmarhubi](https://github.com/kamalmarhubi)
