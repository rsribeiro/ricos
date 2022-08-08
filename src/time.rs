use spin::Mutex;
use core::time::Duration;

static SYSTEM_CLOCK: Mutex<Duration> = Mutex::new(Duration::from_secs(0));

pub (crate) fn system_clock_tick(rate: Duration) {
    let mut clock = SYSTEM_CLOCK.lock();
    *clock += rate;
}

pub fn get_system_uptime() -> Duration {
    *SYSTEM_CLOCK.lock()
}

// #[test_case]
// fn test_system_clock() {
//     let t0 = get_system_uptime();
//     log::trace!("t0 = {:?}", t0);
//     let mut t = get_system_uptime();
//     log::trace!("t = {:?}", t);
//     while t == t0 {
//         t = get_system_uptime();
//         // log::debug!("t = {:?}", t);
//     }
// }
