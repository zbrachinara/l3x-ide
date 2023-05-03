use async_executor::LocalExecutor;
use csv::ReaderBuilder;
use macroquad::prelude::*;
use ndarray_csv::Array2Reader;
use std::collections::HashMap;

use crate::l3x::{L3XParseError, MaybeL3X};

use super::Matrix;

#[cfg(not(target_arch = "wasm32"))]
impl Matrix {
    pub fn start_file_import(&mut self, executor: &mut LocalExecutor) {
        let states = &mut self.future_states;
        if states.read_file.is_none() {
            states.read_file = Some(executor.spawn(async {
                let file = rfd::AsyncFileDialog::new().pick_file().await;
                match file {
                    Some(fi) => Some(fi.read().await),
                    None => None,
                }
            }));
        }
    }

    pub fn try_open_file(&mut self) {
        let states = &mut self.future_states;
        if_chain::if_chain! {
            if let Some(ref task) = states.read_file;
            if task.is_finished();
            if let Some(file) = futures_lite::future::block_on(
                std::mem::take(&mut states.read_file).unwrap()
            );
            then {
                self.import_data(&file);
            }

        }
    }
}

impl Matrix {
    fn import_data(&mut self, data: &[u8]) {
        let mut reader = ReaderBuilder::new().has_headers(false).from_reader(data);
        let Ok(array) = reader.deserialize_array2_dynamic::<String>() else {return};

        if array.is_empty() {
            return; // error
        }

        let mut max_loc = IVec2::ZERO;
        let mut instruction_buffer = HashMap::new();
        if let Err(err) = array
            .columns()
            .into_iter()
            .enumerate()
            .flat_map(|(x, col)| {
                col.into_iter()
                    .enumerate()
                    .map(move |(y, elem)| (ivec2(x as i32, y as i32), elem))
            })
            .try_for_each(|(loc, elem)| {
                log::trace!("trying cell: {elem} at {loc}");
                if let MaybeL3X::Some(l3x) = MaybeL3X::try_from(elem.as_str())? {
                    max_loc = max_loc.max(loc);
                    instruction_buffer.insert(loc, l3x);
                }
                Result::<_, L3XParseError>::Ok(())
            })
        {
            log::warn!("inport failure: {err:?}")
        }

        self.instructions = instruction_buffer;
        self.dims = (max_loc + IVec2::ONE).as_uvec2();
    }
}
