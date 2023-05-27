
const audio_ctx = new AudioContext();
const oscillators = [];

register_plugin = function (importObject) {
  importObject.env.wasm_sound_drop_all = function() {
    oscillators.forEach(oscillator => oscillator.stop())
  }

  importObject.env.wasm_sound_play_a = function() {
    let c_oscillator = audio_ctx.createOscillator()
    c_oscillator.type = 'sine'
    c_oscillator.connect(audio_ctx.destination)
    c_oscillator.start()
    oscillators.push(c_oscillator)
  }
}

// miniquad_add_plugin receive an object with two fields: register_plugin and on_init. Both are functions, both are optional.
miniquad_add_plugin({ register_plugin });
