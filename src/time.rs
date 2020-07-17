use spin::Mutex;
use core::time::Duration;

static SYSTEM_CLOCK: Mutex<Duration> = Mutex::new(Duration::from_secs(0));

pub fn system_clock_tick(rate: Duration) {
    let mut clock = SYSTEM_CLOCK.lock();
    *clock += rate;
}

pub fn get_system_uptime() -> Duration {
    *SYSTEM_CLOCK.lock()
}
