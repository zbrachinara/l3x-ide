
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


register_plugin = function (importObject) {
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
}

// miniquad_add_plugin receive an object with two fields: register_plugin and on_init. Both are functions, both are optional.
miniquad_add_plugin({ register_plugin });
