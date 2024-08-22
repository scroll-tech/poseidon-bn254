use crate::{Fr, State, T};
use std::mem::MaybeUninit;

#[inline(always)]
fn sbox(val: Fr) -> Fr {
    let a = val.square();
    let b = a.square();
    b * val
}

#[inline(always)]
pub(super) fn sbox_inplace(val: &mut Fr) {
    *val = sbox(*val);
}

#[inline(always)]
pub(super) fn fill_state(state: &mut MaybeUninit<State>, val: &Fr) {
    unsafe {
        for i in 0..T {
            (state.as_mut_ptr() as *mut Fr).add(i).write(*val);
        }
    }
}

#[inline(always)]
pub(super) fn set_state(state: &mut State, new_state: &State) {
    state.copy_from_slice(new_state);
}
