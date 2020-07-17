use super::Task;
use alloc::sync::Arc;
use crossbeam_queue::ArrayQueue;
use conquer_once::spin::OnceCell;

pub(crate) static SPAWNER: OnceCell<Spawner> = OnceCell::uninit();

pub fn spawn(task: Task) {
    SPAWNER.get().expect("Task spawner not initialized.").spawn(task);
}

pub(crate) struct Spawner {
    pub shared_task_queue: Arc<ArrayQueue<Task>>
}

impl Spawner {
    pub(crate) fn new() -> Self {
        Spawner {
            shared_task_queue: Arc::new(ArrayQueue::new(100))
        }
    }

    pub(crate) fn clone_shared_task_queue(&self) -> Arc<ArrayQueue<Task>> {
        self.shared_task_queue.clone()
    }

    fn spawn(&self, task: Task) {
        self.shared_task_queue.push(task).expect("queue full");
    }
}