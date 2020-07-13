use std::sync::Arc;
use tokio::sync::RwLock;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorkerState {
    /// The worker has been created but its thread has not yet started
    Starting,
    /// The worker thread is active and it is accepting tasks
    Active,
    /// No more tasks can be submitted, the worker will process any pending tasks
    /// before shutting down
    ShutdownRequested,
    /// All tasks are complete, the worker thread has been stopped
    ShutdownComplete,
}

pub type StateStore = Arc<RwLock<WorkerState>>;
