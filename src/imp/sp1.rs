use crate::{Fr, State, T};
use core::ptr::addr_of_mut;
use std::mem::MaybeUninit;
use sp1_intrinsics::{
    bn254::syscall_bn254_scalar_mul,
    memory::memcpy32,
};

#[inline(always)]
pub(crate) fn sbox_inplace(val: &mut Fr) {
    let mut a = MaybeUninit::<[u32; 8]>::uninit();

    unsafe {
        let ptr = a.as_mut_ptr();
        memcpy32(&val.0, ptr);
        syscall_bn254_scalar_mul(ptr, &val.0);
        syscall_bn254_scalar_mul(ptr, &val.0);
        syscall_bn254_scalar_mul(ptr, &val.0);
        syscall_bn254_scalar_mul(ptr, &val.0);
        memcpy32(ptr, &mut val.0);
    };
}

#[inline(always)]
pub(crate) fn fill_state(state: &mut MaybeUninit<State>, val: &Fr) {
    for i in 0..T {
        unsafe {
            memcpy32(&val.0, addr_of_mut!((*(state.as_mut_ptr().add(i) as *mut Fr)).0));
        }
    }
}

#[inline(always)]
pub(crate) fn set_state(state: &mut State, new_state: &State) {
    unsafe {
        memcpy32(&new_state[0].0, &mut state[0].0);
        memcpy32(&new_state[1].0, &mut state[1].0);
        memcpy32(&new_state[2].0, &mut state[2].0);
    }
}

#[inline(always)]
pub(crate) fn init_state_with_cap_and_msg<'a>(
    state: &'a mut MaybeUninit<State>,
    cap: &Fr,
    msg: &[Fr],
) -> &'a mut State {
    static ZERO: Fr = Fr::zero();

    unsafe {
        let ptr = state.as_mut_ptr() as *mut Fr;
        memcpy32(&cap.0, addr_of_mut!((*ptr).0));
        match msg.len() {
            0 => {
                memcpy32(&ZERO.0, addr_of_mut!((*(ptr.add(1))).0));
                memcpy32(&ZERO.0, addr_of_mut!((*(ptr.add(2))).0));
            },
            1 => {
                memcpy32(&msg[0].0, addr_of_mut!((*(ptr.add(1))).0));
                memcpy32(&ZERO.0,   addr_of_mut!((*(ptr.add(2))).0));
            },
            _ => {
                memcpy32(&msg[0].0, addr_of_mut!((*(ptr.add(1))).0));
                memcpy32(&msg[1].0, addr_of_mut!((*(ptr.add(2))).0));
            },
        }
        state.assume_init_mut()
    }
}

#[inline(always)]
pub(crate) unsafe fn set_fr(dst: *mut Fr, val: &Fr) {
    unsafe {
        memcpy32(&val.0, addr_of_mut!((*dst).0));
    }
}
