class Audio {
  constructor(button, frequency = 1000) {
    this.button = button
    this.muted = true
    this.context = new (window.AudioContext || window.webkitAudioContext)()
    this.oscillator = this.context.createOscillator()

    this.startOscillator(frequency)
    this.button.onclick = (e) => { this.toggle(e.currentTarget) }
  }

  startOscillator(frequency) {
    this.oscillator.type = 'sine'
    this.oscillator.frequency.value = frequency
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

module.exports = Audio;
