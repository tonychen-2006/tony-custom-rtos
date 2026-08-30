use core::sync::atomic::{
    AtomicU32,
    Ordering,
};

static TICKS: AtomicU32 = AtomicU32::new(0);

/// Called by SysTick once every millisecond.
///
/// Returns the new tick count.
pub fn tick() -> u32 {
    TICKS.fetch_add(1, Ordering::Relaxed) + 1
}

/// Milliseconds since the scheduler started.
pub fn now() -> u32 {
    TICKS.load(Ordering::Relaxed)
}