use csv::ReaderBuilder;
use futures_lite::future::block_on;
use macroquad::prelude::*;
use ndarray_csv::Array2Reader;
use rfd::AsyncFileDialog;
use std::collections::HashMap;

use crate::l3x::{L3XParseError, MaybeL3X};

use super::Matrix;

impl<'a> Matrix<'a> {
    pub fn start_file_import(&mut self) {
        if self.read_file.is_none() {
            self.read_file = Some(self.task_executor.spawn(async {
                let file = AsyncFileDialog::new().pick_file().await;
                match file {
                    Some(fi) => Some(fi.read().await),
                    None => None,
                }
            }));
        }
    }

    pub fn try_open_file(&mut self) {
        if_chain::if_chain! {
            if let Some(ref task) = self.read_file;
            if task.is_finished();
            if let Some(file) = block_on(std::mem::take(&mut self.read_file).unwrap());
            then {
                self.import_data(&file);
            }

        }
    }
}

impl<'a> Matrix<'a> {
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
