use crate::{Fr, State, T, Field};
use core::arch::asm;
use std::mem::MaybeUninit;

const GRUMPKIN_FP_ADD: u32 = 0x00_01_01_56;
const GRUMPKIN_FP_MUL: u32 = 0x00_01_01_58;


static ZERO: Fr = Fr::ZERO;
static ONE: Fr = Fr::ONE;

/// a += b
#[inline(always)]
pub(crate) fn add_assign(a: *mut Fr, b: *const Fr) {
   unsafe {
       asm!(
            "ecall",
            in("t0") GRUMPKIN_FP_ADD,
            in("a0") a,
            in("a1") b,
       );
   }
}

/// a *= b
#[inline(always)]
pub(crate) fn mul_assign(a: *mut Fr, b: *const Fr) {
    unsafe {
        asm!(
            "ecall",
            in("t0") GRUMPKIN_FP_MUL,
            in("a0") a,
            in("a1") b,
        );
    }
}

#[inline(always)]
pub(crate) fn set_fr_zero(dst: *mut Fr) {
    // dst *= 0
    mul_assign(dst, &ZERO);
}

#[inline(always)]
pub(crate) fn set_fr(dst: *mut Fr, src: &Fr) {
    unsafe {
        // dst = 0
        // dst += src
        set_fr_zero(dst);
        add_assign(dst, src);
    }
}

#[inline(always)]
pub(crate) fn mul_add_assign(a: &mut Fr, b: &Fr, c: &Fr) {
    // let mut tmp = MaybeUninit::<Fr>::uninit();
    // set_fr(tmp.as_mut_ptr(), b);
    static mut TMP: Fr = Fr::ZERO;
    unsafe {
        set_fr(&mut TMP, b);
        mul_assign(&mut TMP, c);
        add_assign(a, &TMP);
    }
}


#[inline(always)]
pub(crate) fn sbox_inplace(val: &mut Fr) {
    //let mut a = MaybeUninit::<Fr>::uninit();
    static mut TMP: Fr = Fr::ZERO;
    unsafe {
        set_fr(&mut TMP, val);
        mul_assign(&mut TMP, val);
        mul_assign(&mut TMP, val);
        mul_assign(&mut TMP, val);
        mul_assign(&mut TMP, val);
        set_fr(val, &TMP);
    };
}

#[inline(always)]
pub(crate) fn fill_state(state: &mut MaybeUninit<State>, val: &Fr) {
    for i in 0..T {
        unsafe {
            set_fr((state.as_mut_ptr() as *mut Fr).add(i), val);
        }
    }
}

#[inline(always)]
pub(crate) fn set_state(state: &mut State, new_state: &State) {
    unsafe {
        set_fr(&mut state[0], &new_state[0]);
        set_fr(&mut state[1], &new_state[1]);
        set_fr(&mut state[2], &new_state[2]);
    }
}

#[inline(always)]
pub(crate) fn init_state_with_cap_and_msg<'a>(
    state: &'a mut MaybeUninit<State>,
    cap: &Fr,
    msg: &[Fr],
) -> &'a mut State {

    match msg.len() {
        0 => unsafe {
            set_fr(state.as_mut_ptr() as *mut Fr, cap);
            set_fr_zero((state.as_mut_ptr() as *mut Fr).add(1));
            set_fr_zero((state.as_mut_ptr() as *mut Fr).add(2));
        },
        1 => unsafe {
            set_fr(state.as_mut_ptr() as *mut Fr, cap);
            set_fr((state.as_mut_ptr() as *mut Fr).add(1), &msg[0]);
            set_fr_zero((state.as_mut_ptr() as *mut Fr).add(2));
        },
        _ => unsafe {
            set_fr(state.as_mut_ptr() as *mut Fr, cap);
            set_fr((state.as_mut_ptr() as *mut Fr).add(1), &msg[0]);
            set_fr((state.as_mut_ptr() as *mut Fr).add(2), &msg[1]);
        },
    }

    unsafe { state.assume_init_mut() }
}