use conquer_once::spin::{Lazy, OnceCell};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use ps2_mouse::{Mouse as PS2Mouse, MouseState};
use spinning_top::Spinlock;

static WAKER: AtomicWaker = AtomicWaker::new();
pub static MOUSE: Lazy<Spinlock<PS2Mouse>> = Lazy::new(|| Spinlock::new(PS2Mouse::new()));
static MOUSE_PACKET_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub(crate) fn add_mouse_packet(packet: u8) {
    if let Ok(queue) = MOUSE_PACKET_QUEUE.try_get() {
        if let Ok(_) = queue.push(packet) {
            WAKER.wake();
        }
    }
}

pub(crate) fn init() {
    let mut mouse = MOUSE.lock();
    mouse.init().expect("failed to initialize mouse");
    mouse.set_on_complete(on_complete);
}

pub struct MousePacketStream {
    _private: (),
}

impl MousePacketStream {
    pub fn new() -> MousePacketStream {
        // Default mouse settings generate packets at 100 packets per second/
        // This will overflow quickly if the queue is set too low.
        MOUSE_PACKET_QUEUE
            .try_init_once(|| ArrayQueue::new(500))
            .expect("MousePacketStream::new should only be called once");
        MousePacketStream { _private: () }
    }
}

impl Stream for MousePacketStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = MOUSE_PACKET_QUEUE
            .try_get()
            .expect("mouse state queue not initialized");

        if let Some(packet) = queue.pop() {
            return Poll::Ready(Some(packet));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(packet) => {
                WAKER.take();
                Poll::Ready(Some(packet))
            }
            None => Poll::Pending,
        }
    }
}

fn on_complete(mouse_state: MouseState) {
    // log::trace!("{:?}", mouse_state);
    if mouse_state.moved() {
        crate::vga_buffer::cursor_position_delta(mouse_state.get_x(), mouse_state.get_y());
    }
}

pub async fn process_packets() {
    let mut mouse = MOUSE.lock();
    let mut packets = MousePacketStream::new();

    while let Some(packet) = packets.next().await {
        mouse.process_packet(packet);
    }
}
