use csv::{ReaderBuilder, WriterBuilder};
use macroquad::prelude::*;
use ndarray::{ArrayBase, OwnedRepr};
use ndarray_csv::{Array2Reader, Array2Writer};

use std::{borrow::Cow, collections::HashMap};

use crate::{
    l3x::{L3XParseError, MaybeL3X},
    wasync::AsyncContext,
};

use super::Matrix;

impl Matrix {
    pub fn try_import_data(&mut self, ctx: &mut AsyncContext) {
        if let Some(data) = ctx.try_open_file() {
            self.import_data(&data)
        }
    }
}

impl Matrix {
    pub(super) fn export_data(&self) -> Result<Vec<u8>, csv::Error> {
        log::debug!("Beginning file export");
        let mut arr = ArrayBase::<OwnedRepr<_>, _>::from_elem(
            [self.dims.x as usize, self.dims.y as usize],
            Cow::Borrowed(""),
        );

        for (loc @ &IVec2 { x, y }, item) in &self.instructions {
            if loc.cmplt(self.dims.as_ivec2()).all() {
                arr[(y as usize, x as usize)] = Cow::Owned(item.to_string());
            }
        }

        let mut buf_out = Vec::new();
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(&mut buf_out);
        if let Err(e) = writer.serialize_array2(&arr) {
            Err(e)
        } else {
            drop(writer);
            Ok(buf_out)
        }
    }

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
