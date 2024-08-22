use crate::{Fr, State, T};
use core::arch::asm;
use std::mem::MaybeUninit;

const MEMCPY_32: u32 = 0x00_00_01_30;
const MEMCPY_64: u32 = 0x00_00_01_31;
const BN254_SCALAR_MUL: u32 = 0x00_01_01_20;

#[inline(always)]
pub(super) fn sbox_inplace(val: &mut Fr) {
    let mut a = MaybeUninit::<Fr>::uninit();

    unsafe {
        core::arch::asm!(
            "ecall",
            in("t0") MEMCPY_32,
            in("a0") val,
            in("a1") a.as_mut_ptr(),
        );
        core::arch::asm!(
            "ecall",
            in("t0") BN254_SCALAR_MUL,
            in("a0") &mut a,
            in("a1") val,
        );
        core::arch::asm!(
            "ecall",
            in("t0") BN254_SCALAR_MUL,
            in("a0") &mut a,
            in("a1") val,
        );
        core::arch::asm!(
            "ecall",
            in("t0") BN254_SCALAR_MUL,
            in("a0") &mut a,
            in("a1") val,
        );
        core::arch::asm!(
            "ecall",
            in("t0") BN254_SCALAR_MUL,
            in("a0") &mut a,
            in("a1") val,
        );
        core::arch::asm!(
            "ecall",
            in("t0") MEMCPY_32,
            in("a0") &a,
            in("a1") val,
        );
    };
}

#[inline(always)]
pub(super) fn fill_state(state: &mut MaybeUninit<State>, val: &Fr) {
    for i in 0..T {
        unsafe {
            asm!(
                "ecall",
                in("t0") MEMCPY_32,
                in("a0") val,
                in("a1") (state.as_mut_ptr() as *mut Fr).add(i),
            );
        }
    }
}

#[inline(always)]
pub(super) fn set_state(state: &mut State, new_state: &State) {
    unsafe {
        asm!(
            "ecall",
            in("t0") MEMCPY_64,
            in("a0") &new_state[0],
            in("a1") &mut state[0],
        );
        asm!(
            "ecall",
            in("t0") MEMCPY_32,
            in("a0") &new_state[2],
            in("a1") &mut state[2],
        );
    }
}
