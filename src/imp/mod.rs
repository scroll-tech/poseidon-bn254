use crate::{Fr, State, FULL_ROUNDS, MDS, PARTIAL_ROUNDS, ROUND_CONSTANTS, T};
use std::mem::MaybeUninit;
use std::ops::{AddAssign, MulAssign};

#[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
mod host;
#[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
pub(crate) use host::{
    fill_state, init_state_with_cap_and_msg, mul_add_assign, sbox_inplace, set_fr, set_state,
};

#[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
mod sp1;
#[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
pub(crate) use sp1::{
    fill_state, init_state_with_cap_and_msg, mul_add_assign, sbox_inplace, set_fr, set_state,
};

#[inline(always)]
pub fn permute(state: &mut State) {
    const R_F: usize = FULL_ROUNDS / 2;
    const R_P: usize = PARTIAL_ROUNDS;

    let mut new_state = MaybeUninit::<State>::uninit();

    let mut apply_mds = |state: &mut State| {
        fill_state(&mut new_state, &state[0]);

        let new_state = unsafe { new_state.assume_init_mut() };

        // Matrix multiplication
        for i in 0..T {
            new_state[i].mul_assign(&MDS[i][0]);
            for j in 1..T {
                mul_add_assign(&mut new_state[i], &state[j], &MDS[i][j]);
            }
        }

        set_state(state, new_state);
    };

    for i in 0..R_F {
        full_round(state, &ROUND_CONSTANTS[i], &mut apply_mds);
    }
    for i in R_F..R_F + R_P {
        partial_round(state, &ROUND_CONSTANTS[i], &mut apply_mds);
    }
    for i in R_F + R_P..FULL_ROUNDS + PARTIAL_ROUNDS {
        full_round(state, &ROUND_CONSTANTS[i], &mut apply_mds);
    }
}

#[inline(always)]
fn full_round(state: &mut State, rcs: &[Fr; T], mut apply_mds: impl FnMut(&mut State)) {
    for (word, rc) in state.iter_mut().zip(rcs.iter()) {
        word.add_assign(rc);
        sbox_inplace(word);
    }
    apply_mds(state);
}

#[inline(always)]
fn partial_round(state: &mut State, rcs: &[Fr; T], mut apply_mds: impl FnMut(&mut State)) {
    for (word, rc) in state.iter_mut().zip(rcs.iter()) {
        word.add_assign(rc);
    }
    // In a partial round, the S-box is only applied to the first state word.
    sbox_inplace(&mut state[0]);
    apply_mds(state);
}
