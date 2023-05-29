use std::marker::PhantomData;

use crate::matrix::MatrixMode;

extern "C" {
    fn wasm_give_user_file(
        filename_ptr: *const u8,
        filename_len: usize,
        data: *const u8,
        data_len: usize,
    );

    fn wasm_request_file_import();
    fn wasm_file_import_len() -> usize;
    fn wasm_file_import_type() -> usize;
    fn wasm_import_file(buf: *mut u8);
}

/// # Safety
///
/// `data` should be given as a valid utf-8 string
unsafe fn give_user_file(filename: &str, data: &[u8]) {
    let filename_bytes = filename.as_bytes();
    wasm_give_user_file(
        filename_bytes.as_ptr(),
        filename_bytes.len(),
        data.as_ptr(),
        data.len(),
    )
}

fn file_import_type() -> Option<MatrixMode> {
    match unsafe { wasm_file_import_type() } {
        1 => Some(MatrixMode::L3),
        2 => Some(MatrixMode::L3X),
        _ => None,
    }
}

#[derive(Default)]
pub struct AsyncContext<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> AsyncContext<'a> {
    pub fn tick(&mut self) -> bool {
        true
    }

    pub fn try_open_file(&mut self) -> Option<(Vec<u8>, Option<MatrixMode>)> {
        let length = unsafe { wasm_file_import_len() };
        if length > 0 {
            let mut buf = Vec::with_capacity(length);
            let ty = file_import_type();
            unsafe {
                wasm_import_file(buf.as_mut_ptr());
                buf.set_len(length);
            }
            Some((buf, ty))
        } else {
            None
        }
    }

    pub fn try_export_file(&mut self) {} // blank on purpose

    pub fn start_file_import(&mut self) {
        unsafe { wasm_request_file_import() };
    }

    pub fn start_file_export(&mut self, data: Vec<u8>, mode: MatrixMode) {
        let name = match mode {
            MatrixMode::L3 => "l3x-ide_export.l3",
            MatrixMode::L3X => "l3x-ide_export.l3x",
        };
        unsafe { give_user_file(name, &data) }
    }
}
