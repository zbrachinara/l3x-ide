use async_executor::Task;
use rfd::FileHandle;

#[derive(Default)]
pub struct FutureStates {
    pub read_file: Option<Task<Option<Vec<u8>>>>,
    pub write_file: Option<Task<Option<FileHandle>>>,
}
