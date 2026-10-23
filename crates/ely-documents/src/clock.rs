//! How long a test's work keeps the thread busy.

use std::time::Duration;

/// The thread's CPU time, which a busy host does not inflate.
#[cfg(unix)]
pub(crate) fn spent(run: impl FnOnce()) -> Duration {
    use rustix::time::{ClockId, clock_gettime};
    let now = || {
        let at = clock_gettime(ClockId::ThreadCPUTime);
        Duration::new(at.tv_sec as u64, at.tv_nsec as u32)
    };
    let start = now();
    run();
    now() - start
}

/// The wall clock: Windows gives no thread clock without `unsafe`.
#[cfg(not(unix))]
pub(crate) fn spent(run: impl FnOnce()) -> Duration {
    let start = web_time::Instant::now();
    run();
    start.elapsed()
}
