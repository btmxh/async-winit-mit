use std::task::{Context, Poll};

use async_winit::{
    waker::{new_waker, NotifiableUserEvent},
    EventSender,
};
use futures::{pin_mut, Future, StreamExt};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod async_winit;

impl NotifiableUserEvent for () {
    fn new_notify_event() -> Self {}
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let waker = Box::leak(Box::new(new_waker(event_loop.create_proxy())));
    let mut context = Context::from_waker(waker);
    let (sender, receiver) = EventSender::<()>::new();
    let mut async_future = Box::pin(async move {
        let r2 = receiver.clone();
        pin_mut!(r2);
        r2
            .filter(|e| futures::future::ready(matches!(e, Event::Resumed)))
            .next()
            .await;

        let elwt = receiver.get_elwt();
        let _window = Window::new(elwt).expect("unable to create window");

        let r2 = receiver.clone();
        pin_mut!(r2);
        r2
            .filter(|a| {
                futures::future::ready(matches!(
                    a,
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    }
                ))
            })
            .next()
            .await;
        0
    });

    let mut complete = false;
    event_loop.run(move |event, elwt, cf| {
        unsafe { sender.send_event(event, elwt) };
        if !complete {
            if let Poll::Ready(result) = async_future.as_mut().poll(&mut context) {
                complete = true;
                *cf = ControlFlow::ExitWithCode(result);
            }
        }
        sender.clear_event();
    });
}
