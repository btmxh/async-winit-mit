use std::{
    sync::{Arc, Mutex},
    task::{Wake, Waker},
};

use winit::event_loop::EventLoopProxy;

pub trait NotifiableUserEvent: 'static + Send {
    fn new_notify_event() -> Self;
}

pub(crate) struct WakeImpl<T: NotifiableUserEvent> {
    event_loop_proxy: Mutex<EventLoopProxy<T>>,
}

impl<T: NotifiableUserEvent> Wake for WakeImpl<T> {
    fn wake(self: std::sync::Arc<Self>) {
        if let Err(e) = self
            .event_loop_proxy
            .lock()
            .expect("mutex was poisoned")
            .send_event(T::new_notify_event())
        {
            log::error!("Unable to send notify event to event loop: {e}");
        }
    }
}

pub fn new_waker<T: NotifiableUserEvent>(proxy: EventLoopProxy<T>) -> Waker {
    Waker::from(Arc::new(WakeImpl {
        event_loop_proxy: Mutex::new(proxy),
    }))
}
