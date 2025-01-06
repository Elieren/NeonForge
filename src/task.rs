use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;

pub struct Task {
    pub future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }
}
