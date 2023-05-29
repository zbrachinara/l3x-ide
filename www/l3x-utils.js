
// evil copy/paste from wasm_bindgen to get strings into js
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true })
let cachedUint8Memory = null;
function getUint8Memory() {
  if (cachedUint8Memory === null || cachedUint8Memory.byteLength === 0) {
    cachedUint8Memory = new Uint8Array(wasm_memory.buffer);
  }
  return cachedUint8Memory;
}

function get_buf(ptr, len) {
  return getUint8Memory().subarray(ptr, ptr + len)
}
function get_string(ptr, len) {
  return textDecoder.decode(get_buf(ptr, len))
}

// https://stackoverflow.com/a/18197341 CC-BY-SA
function give_user_csv(filename, text) {
  var element = document.createElement('a');
  element.setAttribute('href', 'data:text/csv;charset=utf-8,' + encodeURIComponent(text));
  element.setAttribute('download', filename);

  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
}

const audio_ctx = new AudioContext();
var oscillators = []

const file_input = document.getElementById("file_input")
var stored_file = null
const store_imported_file = function () {
  [f] = this.files // input element is specified to contain exactly one file
  let reader = new FileReader()
  reader.onload = (e) => {
    stored_file = new Uint8Array(e.target.result)
  }
  reader.readAsArrayBuffer(f);
}
file_input.addEventListener("change", store_imported_file, false)

register_plugin = function (importObject) {
  // logging
  importObject.env.wasm_log_error = function (ptr, len) {
    console.error(get_string(ptr, len))
  }
  importObject.env.wasm_log_warn = function (ptr, len) {
    console.warn(get_string(ptr, len))
  }
  importObject.env.wasm_log_info = function (ptr, len) {
    console.info(get_string(ptr, len))
  }
  importObject.env.wasm_log_debug = function (ptr, len) {
    console.log(get_string(ptr, len))
  }
  importObject.env.wasm_log_trace = function (ptr, len) {
    console.debug(get_string(ptr, len))
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

  // files
  importObject.env.wasm_give_user_file = function (filename_ptr, filename_len, data_ptr, data_len) {
    let filename = get_string(filename_ptr, filename_len)
    let data = get_string(data_ptr, data_len)
    give_user_csv(filename, data)
  }
  importObject.env.wasm_request_file_import = function () {
    file_input.click()
  }
  importObject.env.wasm_file_import_len = function () {
    if (!stored_file) {
      return 0
    } else {
      return stored_file.byteLength
    }
  }
  importObject.env.wasm_import_file = function (load_to) {
    getUint8Memory().set(stored_file, load_to)
    stored_file = null
  }
}

miniquad_add_plugin({ register_plugin });
