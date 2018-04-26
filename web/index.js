import { WasmCpu } from './rusty_chip';
import { memory } from './rusty_chip_bg';

import Audio from './audio';
import Display from './display';
import Keypad from './keypad';

const canvas = document.getElementById('screen')
const mute = document.getElementById('mute')

let display = new Display(canvas)
let audio = new Audio(mute)
let keypad = new Keypad()

const cpu = WasmCpu.new();
console.log(cpu)
