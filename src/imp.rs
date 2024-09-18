use crate::{Fr, State, FULL_ROUNDS, MDS, PARTIAL_ROUNDS, ROUND_CONSTANTS, T};

#[inline(always)]
fn sbox(val: Fr) -> Fr {
    let a = val * val;
    let b = a * a;
    b * val
}

#[inline(always)]
pub fn permute(state: &mut State) {
    const R_F: usize = FULL_ROUNDS / 2;
    const R_P: usize = PARTIAL_ROUNDS;

    let mut new_state = *state;

    let mut apply_mds = |state: &mut State| {
        new_state.fill(state[0]);

        // Matrix multiplication
        for i in 0..T {
            new_state[i] = new_state[i] * MDS[i][0];
            for j in 1..T {
                new_state[i] = new_state[i] + state[j] * MDS[i][j];
            }
        }

        *state = new_state;
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
        *word = *word + *rc;
        *word = sbox(*word);
    }
    apply_mds(state);
}

#[inline(always)]
fn partial_round(state: &mut State, rcs: &[Fr; T], mut apply_mds: impl FnMut(&mut State)) {
    for (word, rc) in state.iter_mut().zip(rcs.iter()) {
        *word = *word + *rc;
    }
    // In a partial round, the S-box is only applied to the first state word.
    state[0] = sbox(state[0]);
    apply_mds(state);
}
