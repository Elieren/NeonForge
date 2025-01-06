use alloc::collections::VecDeque;
use crate::task::Task;
use waker_fn::waker_fn;

pub struct Executor {
    tasks: VecDeque<Task>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.tasks.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            let waker = waker_fn(|| {});
            let mut context = core::task::Context::from_waker(&waker);
            if let core::task::Poll::Pending = task.future.as_mut().poll(&mut context) {
                self.tasks.push_back(task);
            }
        }
    }

    pub fn run_until_idle(&mut self) {
        if let Some(mut task) = self.tasks.pop_front() {
            let waker = waker_fn(|| {});
            let mut context = core::task::Context::from_waker(&waker);
            if let core::task::Poll::Pending = task.future.as_mut().poll(&mut context) {
                self.tasks.push_back(task);
            }
        }
    }
}
