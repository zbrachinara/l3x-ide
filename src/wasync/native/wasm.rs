use std::marker::PhantomData;

#[derive(Default)]
pub struct AsyncContext<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> AsyncContext<'a> {
    pub fn tick(&mut self) -> bool {
        true
    }

    pub fn try_open_file(&mut self) -> Option<Vec<u8>> {
        todo!()
    }

    pub fn try_export_file(&mut self) {
        todo!()
    }   

    pub fn start_file_import(&mut self) {
        todo!()
    }

    pub fn start_file_export(&mut self, data: Vec<u8>) {
        todo!()
    }
}
