(function() {
  const BEEP_FREQUENCY = 1000

  class Display {
    constructor(canvas) {
      this.canvas = canvas
      this.context = canvas.getContext('2d')
      this.numPixels = { x: 64, y: 32 }
      this.pixelDimensions = { width: 10, height: 15 }

      this.initContext()
      this.initPixels()
    }

    initContext() {
      this.context.webkitImageSmoothingEnabled = false
      this.context.msImageSmoothingEnabled = false
      this.context.imageSmoothingEnabled = false
      this.context.fillRect(0, 0, this.canvas.width, this.canvas.height);
    }

    initPixels() {
      this.pixels = new Array(this.numPixels.x * this.numPixels.y)
      this.pixels.fill(false)
    }

    flipPixel(i) {
      this.pixels[i] = !this.pixels[i]
    }

    drawPixel(i) {
      let x = i % this.numPixels.x
      let y = Math.floor(i / this.numPixels.x)
      this.context.fillStyle = this.pixels[i] ? 'white' : 'black'
      this.context.fillRect(
        x * this.pixelDimensions.width,
        y * this.pixelDimensions.height,
        this.pixelDimensions.width,
        this.pixelDimensions.height
      )
    }
  }

  class Keypad {
    constructor() {
      this.initListeners()
    }

    initListeners() {
      document.addEventListener('keydown', e => {
        switch(e.key) {
          case "ArrowUp":
            console.log('UP')
            break
          case "ArrowDown":
            console.log('DOWN')
            break
          case "ArrowLeft":
            console.log('LEFT')
            break
          case "ArrowRight":
            console.log('RIGHT')
            break
          case "0":
          case "1":
          case "2":
          case "3":
          case "4":
          case "5":
          case "6":
          case "7":
          case "8":
          case "9":
          case "a":
          case "b":
          case "c":
          case "d":
          case "e":
          case "f":
            console.log(e.key)
            break
          default:
            break
        }
      })

      document.addEventListener('keyup', e => {
        switch(e.key) {
          case "ArrowUp":
            console.log('UP')
            break
          case "ArrowDown":
            console.log('DOWN')
            break
          case "ArrowLeft":
            console.log('LEFT')
            break
          case "ArrowRight":
            console.log('RIGHT')
            break
          case "0":
          case "1":
          case "2":
          case "3":
          case "4":
          case "5":
          case "6":
          case "7":
          case "8":
          case "9":
          case "a":
          case "b":
          case "c":
          case "d":
          case "e":
          case "f":
            console.log(e.key)
            break
          default:
            break
        }
      })
    }
  }

  class Audio {
    constructor(button) {
      this.button = button
      this.muted = true
      this.context = new (window.AudioContext || window.webkitAudioContext)()
      this.oscillator = this.context.createOscillator()

      this.startOscillator()
      this.button.onclick = (e) => { this.toggle(e.currentTarget) }
    }

    startOscillator() {
      this.oscillator.type = 'sine'
      this.oscillator.frequency.value = BEEP_FREQUENCY
      this.oscillator.start()
    }

    mute() {
      if (!this.muted) {
        this.muted = true
        this.oscillator.disconnect(this.context.destination)
      }
    }

    unmute() {
      if (this.muted) {
        this.muted = false
        this.oscillator.connect(this.context.destination)
      }
    }

    toggle(button) {
      let mutedIcon = button.querySelector('.muted')
      let unmutedIcon = button.querySelector('.unmuted')
      mutedIcon.hidden = !mutedIcon.hidden
      unmutedIcon.hidden = !unmutedIcon.hidden
      this.muted ? this.unmute() : this.mute()
    }
  }

  const canvas = document.getElementById('screen')
  const mute = document.getElementById('mute')

  const display = new Display(canvas)
  const audio = new Audio(mute)
  const keypad = new Keypad()
})()
