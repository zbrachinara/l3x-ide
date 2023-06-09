use std::{ffi::OsStr, fs::OpenOptions, io::Write};

use async_executor::Task;
use rfd::FileHandle;

use crate::matrix::MatrixMode;

type ReadFileOutput = (Vec<u8>, Option<MatrixMode>);

#[derive(Default)]
pub struct AsyncContext<'a> {
    executor: async_executor::LocalExecutor<'a>,
    read_file: Option<Task<Option<ReadFileOutput>>>,
    write_file: Option<Task<Option<(FileHandle, &'static str)>>>,
    pending_data: Option<Vec<u8>>,
}

fn l3x_extension(ext: &OsStr) -> Option<MatrixMode> {
    if ext == "l3" {
        Some(MatrixMode::L3)
    } else if ext == "l3x" {
        Some(MatrixMode::L3X)
    } else {
        None
    }
}

impl<'a> AsyncContext<'a> {
    pub fn tick(&mut self) -> bool {
        self.executor.try_tick()
    }

    pub fn start_file_import(&mut self) {
        if self.read_file.is_none() {
            self.read_file = Some(self.executor.spawn(async {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("L3X File", &["l3x", "l3"])
                    .add_filter("CSV", &["csv"])
                    .pick_file()
                    .await;
                match file {
                    Some(fi) => Some((
                        fi.read().await,
                        fi.path().extension().and_then(l3x_extension),
                    )),
                    None => None,
                }
            }));
        }
    }

    pub fn start_file_export(&mut self, data: Vec<u8>, mode: MatrixMode) {
        let (filter_name, extension) = match mode {
            MatrixMode::L3 => ("L3", "l3"),
            MatrixMode::L3X => ("L3X", "l3x"),
        };
        if self.write_file.is_none() {
            self.write_file = Some(self.executor.spawn(async move {
                let path = rfd::AsyncFileDialog::new()
                    .add_filter(filter_name, &[extension])
                    .save_file()
                    .await;
                path.map(|p| (p, extension))
            }));
            self.pending_data = Some(data);
        }
    }

    pub fn try_open_file(&mut self) -> Option<(Vec<u8>, Option<MatrixMode>)> {
        if_chain::if_chain! {
            if let Some(ref task) = self.read_file;
            if task.is_finished();
            if let Some(file) = futures_lite::future::block_on(
                std::mem::take(&mut self.read_file).unwrap()
            );
            then {
                Some(file)
            } else {
                None
            }
        }
    }

    pub fn try_export_file(&mut self) {
        if_chain::if_chain! {
            if let Some(ref task) = self.write_file;
            if task.is_finished();
            if let Some((handle, extension)) = futures_lite::future::block_on(
                std::mem::take(&mut self.write_file).unwrap()
            );
            then {
                let mut path = handle.path().to_path_buf();
                path.set_extension(extension);

                let mut file = match OpenOptions::new()
                    .truncate(false)
                    .write(true)
                    .create(true)
                    .open(path) {
                    Ok(fi) => fi,
                    Err(e) => {
                        log::error!("File could not be opnened: {e}");
                        return;
                    }
                };

                let _ = file.write(&std::mem::take(&mut self.pending_data).unwrap());
            }
        }
    }
}
