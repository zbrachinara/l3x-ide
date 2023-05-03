use async_executor::Task;

#[derive(Default)]
pub struct FutureStates {
    pub read_file: Option<Task<Option<Vec<u8>>>>,
}
