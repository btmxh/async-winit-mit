use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::Stream;
use winit::{event::Event, event_loop::EventLoopWindowTarget};

// change this to Arc and Mutex/RwLock to make it works in multithread idk
pub type SharedPtr<T> = Rc<T>;
pub type Lock<T> = RefCell<T>;

pub mod waker;

struct Shared<T: 'static> {
    event: Lock<Option<Event<'static, T>>>,
    elwt: Lock<Option<&'static EventLoopWindowTarget<T>>>,
    // control_flow: Lock<ControlFlow>,
    waker: Lock<Option<Waker>>,
}

pub struct EventSender<T: 'static> {
    shared: SharedPtr<Shared<T>>,
}

#[derive(Clone)]
pub struct EventReceiver<T: 'static> {
    shared: SharedPtr<Shared<T>>,
}

impl<T: 'static> Shared<T> {
    fn new() -> Self {
        Self {
            event: RefCell::new(None),
            elwt: RefCell::new(None),
            // control_flow: RefCell::new(ControlFlow::Poll),
            waker: RefCell::new(None),
        }
    }
}

impl<T: 'static> EventSender<T> {
    pub fn new() -> (Self, EventReceiver<T>) {
        let shared = SharedPtr::new(Shared::new());
        (
            Self {
                shared: shared.clone(),
            },
            EventReceiver { shared },
        )
    }

    pub unsafe fn send_event<'a>(&self, event: Event<'a, T>, elwt: &EventLoopWindowTarget<T>) {
        self.shared.event.replace(Some(
            std::mem::transmute::<Event<'a, T>, Event<'static, T>>(event),
        ));
        self.shared.elwt.replace(Some(std::mem::transmute::<
            &EventLoopWindowTarget<T>,
            &'static EventLoopWindowTarget<T>,
        >(elwt)));
    }

    pub fn clear_event(&self) {
        self.shared.event.replace(None);
    }
}

impl<T: 'static> Stream for EventReceiver<T> {
    type Item = Event<'static, T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(event) = self.shared.event.borrow_mut().take() {
            Poll::Ready(Some(event))
        } else {
            self.shared.waker.replace(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}

impl<T: 'static> EventReceiver<T> {
    pub(crate) fn get_elwt(&self) -> &EventLoopWindowTarget<T> {
        self.shared
            .elwt
            .borrow()
            .as_ref()
            .expect("invalid get_elwt call")
    }
}
