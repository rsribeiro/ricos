use core::{pin::Pin, task::{Poll, Context}};
use futures_util::{future::Future, task::AtomicWaker};
use spin::Mutex;
use core::time::Duration;

static TIMER: Mutex<Duration> = Mutex::new(Duration::from_secs(0));
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn sleep_task_tick(rate: Duration) {
    let mut timer = TIMER.lock();
    if *timer > Duration::from_secs(0) {
        *timer = (*timer).checked_sub(rate).unwrap_or(Duration::from_secs(0));
    } else {
        WAKER.wake();
    }
}

pub struct Sleep {
    _private: (),
}

impl Sleep {
    pub fn new(duration: Duration) -> Self {
        setup_sleep(duration);
        Sleep { _private: () }
    }
}

fn setup_sleep(duration: Duration) {
    let mut timer = TIMER.lock();
    *timer = duration;
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if *TIMER.lock() == Duration::from_secs(0) {
            return Poll::Ready(());
        }

        WAKER.register(&cx.waker());
        if *TIMER.lock() == Duration::from_secs(0) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
