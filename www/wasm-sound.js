
const audio_ctx = new AudioContext();
var oscillators = []

register_plugin = function (importObject) {
  importObject.env.wasm_sound_drop_all = function () {
    oscillators.forEach(oscillator => oscillator.stop())
    oscillators = []
  }

  importObject.env.wasm_sound_play = function (frequency, volume) {
    let osc = audio_ctx.createOscillator()
    osc.type = 'sine'
    osc.frequency.value = frequency

    let vol = audio_ctx.createGain()
    vol.gain.value = volume

    osc.connect(vol)
    vol.connect(audio_ctx.destination)
    osc.start()

    oscillators.push(osc)
  }
}

// miniquad_add_plugin receive an object with two fields: register_plugin and on_init. Both are functions, both are optional.
miniquad_add_plugin({ register_plugin });
