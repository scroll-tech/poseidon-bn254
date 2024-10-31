use crate::{Fr, State, T};
use sp1_intrinsics::{
    bn254::{syscall_bn254_scalar_mac, syscall_bn254_scalar_mul},
    memory::{memcpy32, memcpy64},
};
use std::mem::MaybeUninit;

#[inline(always)]
pub(crate) fn sbox_inplace(val: &mut Fr) {
    let mut a = MaybeUninit::<Fr>::uninit();

    unsafe {
        let ptr = a.as_mut_ptr();
        memcpy32(val, ptr);
        syscall_bn254_scalar_mul(ptr, val);
        syscall_bn254_scalar_mul(ptr, val);
        syscall_bn254_scalar_mul(ptr, val);
        syscall_bn254_scalar_mul(ptr, val);
        memcpy32(ptr, val);
    };
}

#[inline(always)]
pub(crate) fn fill_state(state: &mut MaybeUninit<State>, val: &Fr) {
    let ptr = state.as_mut_ptr() as *mut Fr;
    for i in 0..T {
        unsafe {
            memcpy32(val, ptr.add(i));
        }
    }
}

#[inline(always)]
pub(crate) fn set_state(state: &mut State, new_state: &State) {
    unsafe {
        memcpy32(&new_state[0], &mut state[0]);
        memcpy32(&new_state[1], &mut state[1]);
        memcpy32(&new_state[2], &mut state[2]);
    }
}

#[inline(always)]
pub(crate) fn init_state_with_cap_and_msg<'a>(
    state: &'a mut MaybeUninit<State>,
    cap: &Fr,
    msg: &[Fr],
) -> &'a mut State {
    static ZERO_TWO: [Fr; 2] = [Fr::zero(), Fr::zero()];

    unsafe {
        let ptr = state.as_mut_ptr() as *mut Fr;
        memcpy32(cap, ptr);
        match msg.len() {
            0 => {
                memcpy64(ZERO_TWO.as_ptr(), ptr.add(1));
            }
            1 => {
                memcpy32(msg.as_ptr(), ptr.add(1));
                memcpy32(ZERO_TWO.as_ptr(), ptr.add(2));
            }
            _ => {
                memcpy64(msg.as_ptr(), ptr.add(1));
            }
        }
        state.assume_init_mut()
    }
}

#[inline(always)]
pub(crate) unsafe fn set_fr(dst: *mut Fr, val: &Fr) {
    unsafe {
        memcpy32(val, dst);
    }
}

#[inline(always)]
pub(crate) fn mul_add_assign(dst: &mut Fr, a: &Fr, b: &Fr) {
    unsafe {
        syscall_bn254_scalar_mac(dst, a, b);
    }
}
