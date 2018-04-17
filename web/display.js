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

module.exports = Display;
