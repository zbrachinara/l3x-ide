use async_executor::{LocalExecutor, Task};

#[derive(Default)]
pub struct FutureStates<'a> {
    pub task_executor: LocalExecutor<'a>,
    pub read_file: Option<Task<Option<Vec<u8>>>>,
}
