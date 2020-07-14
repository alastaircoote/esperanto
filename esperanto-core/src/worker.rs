use esperanto_traits::JSContext;
use esperanto_traits::errors::{JSEnvError, JSConversionError};
use crate::worker_state::{StateStore, WorkerState};
use log::info;
use std::sync::Arc;
use std::thread;
use std::thread::ThreadId;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::sync::Mutex;

type RunloopOperation<Runtime> = Box<dyn Send + FnOnce(&mut Runtime) -> ()>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum WorkerError {
    CouldNotCreateWorker,
    OperationEnqueueFailed,
    OperationResultFetchFailed,
    CannotEnqueueInThisState,
    NotInActiveState,
    WorkerStoppedUnexpectedly,
    InternalRuntimeError(JSEnvError),
    ConversionError(JSConversionError)
}

pub struct Worker<Runtime: JSContext + 'static> {
    sender: mpsc::UnboundedSender<RunloopOperation<Runtime>>,
    pub state: StateStore,
    #[allow(dead_code)] // It's used in tests!
    thread_id: ThreadId,
    shutdown_complete: Arc<Mutex<Option<WorkerError>>>
}

impl<Runtime: JSContext + 'static> Worker<Runtime> {
    pub async fn new() -> Result<Self, WorkerError> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<RunloopOperation<Runtime>>();

        // We use a oneshot channel to hold off returning the worker until we know it is active. That way
        // we avoid dispatching operations to a worker that can't receive them.
        let (ready_tx, ready_rx) = oneshot::channel();

        // We use a mutex to signal to the outside world when the worker has finished executing, and send an
        // error should one have occurred.
        let shutdown_complete = Arc::new(Mutex::new(None));

        // We set an initial state of starting, which then gets modified by the task once it starts
        let state = Arc::new(RwLock::new(WorkerState::Starting));

        let state_in_task = state.clone();
        let shutdown_complete_in_task = shutdown_complete.clone();

        info!(target: "worker", "Creating new worker, spinning up task...");

        std::thread::Builder::new()
            .name("JS Worker".to_string())
            .spawn(move || {
            let mut runtime = tokio::runtime::Builder::new()
                .basic_scheduler()
                .core_threads(1)
                .max_threads(1)
                .build()
                .unwrap();

            runtime.block_on(async move {
                info!(target: "worker", "New tokio task successfully. Creating runtime and runloop.");
                let state = state_in_task;

                // We lock the mutex while the worker is running
                let mut shutdown_complete_lock = shutdown_complete_in_task.lock().await;
    
                let mut runtime = Runtime::new();
    
                *state.write().await = WorkerState::Active;
    
                // This feels kind of janky but I'm mapping the result of this send into an optional. If the send
                // has failed it'll be None, and we try to send a message to the shutdown signaller that
                // the worker failed.
                match ready_tx.send(thread::current().id()).map_or(None, |r| {Some(r)}) {
                    None => {
                        // Of course, *this* could fail too! In that case we just unwrap and let the worker thread
                        // panic because it's not really clear what we should do.

                        *shutdown_complete_lock = Some(WorkerError::CouldNotCreateWorker);
                        return
                    }
                    Some(_) => {}
                }
    
                loop {
                    let val = *state.read().await;
                    match val {
                        WorkerState::Starting => {
                            // this shouldn't ever happen!
                            break;
                        }
                        WorkerState::Active => {
                            info!("Waiting on a runloop operation...");
                            match receiver.recv().await {
                                None => {
                                    // All the senders have disconnected, so we start the shutdown
                                    *state.write().await = WorkerState::ShutdownRequested;
                                }
                                Some(operation) => {
                                    // We've received an operation to run on-thread, so let's go
                                    // ahead and execute it.
                                    info!("Received operation, running...");
                                    operation(&mut runtime);
                                }
                            }
                        }
                        WorkerState::ShutdownRequested => {
                            match receiver.try_recv() {
                                Err(_) => {
                                    // this covers two states: all senders are disconnected or the queue
                                    // is empty. In either case shutdown is complete.
                                    *state.write().await = WorkerState::ShutdownComplete;
                                }
                                Ok(operation) => {
                                    operation(&mut runtime);
                                }
                            }
                        }
                        WorkerState::ShutdownComplete => {
                            break;
                        }
                    }
                }
                *shutdown_complete_lock = None;
            });
            
        }).map_err(|_| {WorkerError::CouldNotCreateWorker})?;

        
        // Now we wait on a read lock on the state. It means we ensure the runtime has been successfully
        // created before we return, avoiding a situation where we might dispatch an operation to a worker
        // that can't actually be started.
        let thread_id = ready_rx
            .await
            .map_err(|_| WorkerError::CouldNotCreateWorker)?;

        Ok(Worker {
            sender,
            state,
            thread_id,
            shutdown_complete
        })
    }

    pub async fn get_state(&self) -> WorkerState {
        *self.state.read().await
    }

    /// Add an operation to the run loop queue
    pub async fn enqueue<T: Send + 'static, F: (FnOnce(&mut Runtime) -> T) + Send + 'static>(
        &self,
        operation: F,
    ) -> Result<T, WorkerError> {
        match *self.state.read().await {
            WorkerState::Active => {
                // Because we can't send generics over our master channel we set it to accept
                // closures with no return value, and instead send our return values over a
                // oneshot channel.

                let (result_tx, result_rx) = oneshot::channel();

                self.sender
                    .send(Box::new(move |r| {
                        let result = operation(r);
                        // Not sure if we want a more graceful way of doing this? But if our
                        // result sender doesn't work it's not really clear what else we can do!
                        // At least once this completes the tx will be discarded and the rx will
                        // receive a disconnection error.
                        result_tx.send(result).unwrap_or(());
                    }))
                    .map_err(|_| WorkerError::OperationEnqueueFailed)?;

                result_rx
                    .await
                    .map_err(|_| WorkerError::OperationResultFetchFailed)
            }
            _ => Err(WorkerError::CannotEnqueueInThisState),
        }
    }

    pub async fn request_shutdown(&self) -> Result<(), WorkerError> {
        let mut writer = self.state.write().await;
        match *writer {
            WorkerState::Active => {

                // If the state is what we're expecting then change it to request the shutdown
                *writer = WorkerState::ShutdownRequested;

                // This feels dumb! But more likely than not the runloop is currently awaiting an instruction
                // so it won't notice the status change. If we enqueue a no-op it'll mean the runloop loop {} starts
                // again and it'll read the state change. I wonder if there's a better way to do this?
                self.sender.send(Box::new(|_| {})).map_err(|_| {WorkerError::OperationEnqueueFailed})?;

                Ok(())
            }
            _ => Err(WorkerError::NotInActiveState),
        }
    }

    pub async fn wait_for_shutdown(&self) -> Result<(), WorkerError> {
        match *self.shutdown_complete.lock().await {
            None => {
                // Worker shut down successfully, great!
                Ok(())
            }
            Some(err) => Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::DummyJSContext;

    // type EmptyRuntime = AlwaysReturnRuntime<EmptyJSValue>;

    #[tokio::test]
    async fn workers_run_on_the_right_thread() {
        // This is mostly a sanity check to make sure that tokio tasks work the way I think they do
        let worker = Worker::<DummyJSContext>::new().await.unwrap();
        let result = worker
            .enqueue(|_| return std::thread::current().id())
            .await
            .unwrap();
        assert_eq!(result, worker.thread_id);
        // double check it's not because everything is running on the same thread.
        assert_ne!(std::thread::current().id(), result);
    }

    #[tokio::test]
    async fn cannot_enqueue_when_shutdown() {
        let worker = Worker::<DummyJSContext>::new().await.unwrap();
        worker.request_shutdown().await.unwrap();
        let err = worker.enqueue(|_| {}).await.unwrap_err();
        assert_eq!(err, WorkerError::CannotEnqueueInThisState);

        worker.wait_for_shutdown().await.unwrap();

        assert_eq!(*worker.state.read().await, WorkerState::ShutdownComplete);
    }

    #[tokio::test]
    async fn states_are_correct_during_lifecycle() {
        let worker = Worker::<DummyJSContext>::new().await.unwrap();
        assert_eq!(*worker.state.read().await, WorkerState::Active);
        worker.request_shutdown().await.unwrap();
        assert_eq!(*worker.state.read().await, WorkerState::ShutdownRequested);
        worker.wait_for_shutdown().await.unwrap();
        assert_eq!(*worker.state.read().await, WorkerState::ShutdownComplete);
    }
}
