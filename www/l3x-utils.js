
// evil copy/paste from wasm_bindgen to get strings into js
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true })
let cachedUint8Memory = null;
function getUint8Memory() {
  if (cachedUint8Memory === null || cachedUint8Memory.byteLength === 0) {
    cachedUint8Memory = new Uint8Array(wasm_memory.buffer);
  }
  return cachedUint8Memory;
}
textDecoder.decode()
function getString(ptr, len) {
  return textDecoder.decode(getUint8Memory().subarray(ptr, ptr + len))
}

const audio_ctx = new AudioContext();
var oscillators = []

register_plugin = function (importObject) {
  // logging
  importObject.env.wasm_log_error = function (ptr, len) {
    console.error(getString(ptr, len))
  }
  importObject.env.wasm_log_warn = function (ptr, len) {
    console.warn(getString(ptr, len))
  }
  importObject.env.wasm_log_info = function (ptr, len) {
    console.info(getString(ptr, len))
  }
  importObject.env.wasm_log_debug = function (ptr, len) {
    console.log(getString(ptr, len))
  }
  importObject.env.wasm_log_trace = function (ptr, len) {
    console.debug(getString(ptr, len))
  }

  // sound
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

miniquad_add_plugin({ register_plugin });
