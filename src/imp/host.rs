use crate::{Fr, State};
use std::mem::MaybeUninit;

#[inline(always)]
pub(crate) fn mul_add_assign(a: &mut Fr, b: &Fr, c: &Fr) {
    *a += b * c;
}

#[inline(always)]
fn sbox(val: Fr) -> Fr {
    let a = val.square();
    let b = a.square();
    b * val
}

#[inline(always)]
pub(crate) fn sbox_inplace(val: &mut Fr) {
    *val = sbox(*val);
}

#[inline(always)]
pub(crate) fn init_state_with_cap_and_msg<'a>(
    state: &'a mut MaybeUninit<State>,
    cap: &Fr,
    msg: &[Fr],
) -> &'a mut State {
    match msg.len() {
        0 => state.write([*cap, Fr::zero(), Fr::zero()]),
        1 => state.write([*cap, msg[0], Fr::zero()]),
        _ => state.write([*cap, msg[0], msg[1]]),
    }
}

#[inline(always)]
pub(crate) unsafe fn set_fr(dst: *mut Fr, val: &Fr) {
    dst.write(*val);
}
