// copy from https://github.com/rust-lang/log/blob/master/src/lib.rs#L452

use std::sync::atomic::Ordering;

pub static mut ZKVM_HINT_HOOK: &dyn Fn([u8; 32]) = &|_| {};
pub static STATE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Debug)]
pub struct SetZkvmHintHookError(pub(super) ());

pub const UNINITIALIZED: usize = 0;
pub const INITIALIZING: usize = 1;
pub const INITIALIZED: usize = 2;

// copy from https://github.com/rust-lang/log/blob/master/src/lib.rs#L1400
#[cfg(all(feature = "zkvm-hint", target_has_atomic = "ptr"))]
pub fn set_zkvm_hint_hook<F>(make_callback: F) -> Result<(), SetZkvmHintHookError>
where
    F: FnOnce() -> &'static dyn Fn([u8; 32]),
{
    use std::sync::atomic::Ordering;
    match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::Acquire,
        Ordering::Relaxed,
    ) {
        Ok(UNINITIALIZED) => {
            unsafe {
                ZKVM_HINT_HOOK = make_callback();
            }
            STATE.store(INITIALIZED, Ordering::Release);
            Ok(())
        }
        Err(INITIALIZING) => {
            while STATE.load(Ordering::Relaxed) == INITIALIZING {
                std::hint::spin_loop();
            }
            Err(SetZkvmHintHookError(()))
        }
        _ => Err(SetZkvmHintHookError(())),
    }
}

#[inline]
pub fn hint(result: [u8; 32]) {
    let hook = if STATE.load(Ordering::Acquire) != INITIALIZED {
        &|_| {}
    } else {
        unsafe { ZKVM_HINT_HOOK }
    };
    hook(result);
}
