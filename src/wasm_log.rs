use log::{Level, LevelFilter, Log, SetLoggerError};

extern "C" {
    fn wasm_log_error(ptr: *const u8, len: usize);
    fn wasm_log_warn(ptr: *const u8, len: usize);
    fn wasm_log_info(ptr: *const u8, len: usize);
    fn wasm_log_debug(ptr: *const u8, len: usize);
    fn wasm_log_trace(ptr: *const u8, len: usize);
}

/// Even simpler copy of SimpleLogger in order to circumvent incompatibility between `wasm_bindgen`
/// and `miniquad` (macroquad devs pls fix ur stuf im bad at unsafe)
pub struct WasmLogger {
    default_level: LevelFilter,
    module_levels: Vec<(String, LevelFilter)>,
}

impl Default for WasmLogger {
    fn default() -> Self {
        Self {
            default_level: LevelFilter::Error,
            module_levels: Default::default(),
        }
    }
}

impl Log for WasmLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        &metadata.level().to_level_filter()
            <= self
                .module_levels
                .iter()
                /* At this point the Vec is already sorted so that we can simply take
                 * the first match
                 */
                .find(|(name, _level)| metadata.target().starts_with(name))
                .map(|(_name, level)| level)
                .unwrap_or(&self.default_level)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let msg = record.args().to_string();
            let len = msg.len();
            let ptr = msg.as_ptr();
            std::mem::forget(msg);
            unsafe {
                match record.level() {
                    Level::Error => wasm_log_error(ptr, len),
                    Level::Warn => wasm_log_warn(ptr, len),
                    Level::Info => wasm_log_info(ptr, len),
                    Level::Debug => wasm_log_debug(ptr, len),
                    Level::Trace => wasm_log_trace(ptr, len),
                }
            }
        }
    }

    fn flush(&self) {}
}

#[allow(dead_code)]
impl WasmLogger {
    pub fn with_module_level(mut self, target: &str, level: LevelFilter) -> Self {
        self.module_levels.push((target.to_string(), level));
        self
    }

    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.default_level = level;
        self
    }
}

impl WasmLogger {
    pub fn init(mut self) -> Result<(), SetLoggerError> {
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());
        let max_level = self
            .module_levels
            .iter()
            .map(|(_name, level)| level)
            .copied()
            .max();
        let max_level = max_level
            .map(|lvl| lvl.max(self.default_level))
            .unwrap_or(self.default_level);
        log::set_max_level(max_level);
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }
}
