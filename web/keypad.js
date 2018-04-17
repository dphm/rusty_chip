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

module.exports = Keypad;
