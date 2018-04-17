const Audio = require('./audio.js')
const Display = require('./display.js')
const Keypad = require('./keypad.js')

const canvas = document.getElementById('screen')
const mute = document.getElementById('mute')

let display = new Display(canvas)
let audio = new Audio(mute)
let keypad = new Keypad()
