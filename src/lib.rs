#![allow(clippy::needless_range_loop)]
#![allow(clippy::op_ref)]
#![allow(unexpected_cfgs)]

use itertools::Itertools;
use std::mem::MaybeUninit;
use std::ops::AddAssign;

#[cfg(feature = "bn254")]
pub use bn254::ff::{Field, PrimeField};
#[cfg(feature = "halo2curves_v1")]
pub use halo2curves_v1::ff::{Field, PrimeField};
#[cfg(feature = "halo2curves_v3")]
pub use halo2curves_v3::ff::{Field, PrimeField};

mod constants;
mod imp;
#[cfg(all(
    not(target_os = "zkvm"),
    not(target_vendor = "succinct"),
    feature = "zkvm-hint"
))]
mod zkvm_hints;

#[cfg(feature = "bn254")]
pub use bn254::Fr;
#[cfg(feature = "halo2curves_v1")]
pub use halo2curves_v1::bn256::Fr;
#[cfg(feature = "halo2curves_v3")]
pub use halo2curves_v3::bn256::Fr;

#[cfg(all(
    not(target_os = "zkvm"),
    not(target_vendor = "succinct"),
    feature = "zkvm-hint"
))]
pub use zkvm_hints::set_zkvm_hint_hook;

pub(crate) use constants::*;

pub(crate) type State = [Fr; T];
pub(crate) type Mds = [[Fr; T]; T];

pub fn hash_with_domain(inp: &[Fr; 2], domain: Fr) -> Fr {
    #[cfg(all(target_os = "zkvm", target_vendor = "succinct", feature = "zkvm-hint"))]
    return Fr::from_repr_vartime(sp1_lib::io::read_vec().try_into().unwrap()).unwrap();

    if inp[1].is_zero_vartime() && inp[0].is_zero_vartime() && domain.is_zero_vartime() {
        return EMPTY_HASH;
    }
    let mut state = MaybeUninit::<State>::uninit();
    let state = imp::init_state_with_cap_and_msg(&mut state, &domain, inp);
    imp::permute(state);

    #[cfg(all(
        not(target_os = "zkvm"),
        not(target_vendor = "succinct"),
        feature = "zkvm-hint"
    ))]
    zkvm_hints::hint(state[0].to_repr());

    state[0]
}

pub fn hash_msg(msg: &[Fr], cap: Option<u128>) -> Fr {
    debug_assert_eq!(RATE, 2);

    if msg.is_empty() && cap.map(|c| c == 0).unwrap_or(true) {
        return EMPTY_HASH;
    }

    #[cfg(all(target_os = "zkvm", target_vendor = "succinct", feature = "zkvm-hint"))]
    return Fr::from_repr_vartime(sp1_lib::io::read_vec().try_into().unwrap()).unwrap();

    let cap = cap.map(Fr::from_u128).unwrap_or_else(|| {
        // trick here since msg.len() won't exceed u64::MAX
        // msg.len() * (1 << 64) = msg.len() << 64
        Fr::from_raw([0, msg.len() as u64, 0, 0])
    });

    let mut state = MaybeUninit::<State>::uninit();

    let state = imp::init_state_with_cap_and_msg(&mut state, &cap, msg);
    imp::permute(state);

    if msg.len() > 2 {
        for chunk in msg.chunks(RATE).skip(1) {
            if chunk.len() == RATE {
                state[1].add_assign(&chunk[0]);
                state[2].add_assign(&chunk[1]);
                imp::permute(state);
            } else {
                state[1].add_assign(&chunk[0]);
                imp::permute(state);
            }
        }
    };

    #[cfg(all(
        not(target_os = "zkvm"),
        not(target_vendor = "succinct"),
        feature = "zkvm-hint"
    ))]
    zkvm_hints::hint(state[0].to_repr());

    state[0]
}

pub fn hash_code(code: &[u8]) -> [u8; 32] {
    if code.is_empty() {
        return EMPTY_HASH_BYTES;
    }

    #[cfg(all(target_os = "zkvm", target_vendor = "succinct", feature = "zkvm-hint"))]
    return sp1_lib::io::read_vec().try_into().unwrap();

    let mut msg = code.chunks(POSEIDON_HASH_BYTES_IN_FIELD).map(|chunk| {
        let mut be_bytes = [0u8; 32];
        be_bytes[1..1 + chunk.len()].copy_from_slice(chunk);
        be_bytes.reverse();
        Fr::from_bytes(&be_bytes).unwrap()
    });

    let cap = Fr::from_raw([0, code.len() as u64, 0, 0]);

    let mut bytes = match msg.len() {
        // Safety: we know that the iterator is not empty
        0 => unsafe { std::hint::unreachable_unchecked() },
        1 => {
            let hash =
                hash_with_domain(&[unsafe { msg.next().unwrap_unchecked() }, Fr::zero()], cap);
            hash.to_repr()
        }
        _ => {
            let mut state = MaybeUninit::<State>::uninit();
            let state = unsafe {
                imp::set_fr(state.as_mut_ptr() as *mut Fr, &cap);
                imp::set_fr(
                    (state.as_mut_ptr() as *mut Fr).add(1),
                    &msg.next().unwrap_unchecked(),
                );
                imp::set_fr(
                    (state.as_mut_ptr() as *mut Fr).add(2),
                    &msg.next().unwrap_unchecked(),
                );
                state.assume_init_mut()
            };
            imp::permute(state);

            for mut chunk in msg.chunks(RATE).into_iter() {
                let a = chunk.next().unwrap();
                if let Some(b) = chunk.next() {
                    state[1].add_assign(a);
                    state[2].add_assign(b);
                    imp::permute(state);
                } else {
                    state[1].add_assign(a);
                    imp::permute(state);
                }
            }
            state[0].to_repr()
        }
    };

    bytes[0..8].reverse();
    bytes[8..16].reverse();
    bytes[16..24].reverse();
    bytes[24..32].reverse();
    let mut result = [0u8; 32];
    result[24..32].copy_from_slice(&bytes[0..8]);
    result[16..24].copy_from_slice(&bytes[8..16]);
    result[8..16].copy_from_slice(&bytes[16..24]);
    result[0..8].copy_from_slice(&bytes[24..32]);

    #[cfg(all(
        not(target_os = "zkvm"),
        not(target_vendor = "succinct"),
        feature = "zkvm-hint"
    ))]
    zkvm_hints::hint(result);

    result
}

#[cfg(all(test, feature = "halo2curves_v1"))]
mod tests {
    use super::*;
    use ethers_core::types::U256;
    use itertools::iproduct;
    use poseidon_base::hash::{Hashable, MessageHashable, HASHABLE_DOMAIN_SPEC};
    use std::array;

    #[test]
    fn test_empty_hash() {
        let inp = [Fr::zero(), Fr::zero()];
        let domain = Fr::zero();
        let result = hash_with_domain(&inp, domain);
        let expected = Fr::hash_with_domain(inp, domain);
        assert_eq!(result, expected);

        let result = hash_msg(&[], Some(0));
        assert_eq!(result, expected);

        let result = hash_msg(&[], None);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_with_domain() {
        let inp = [Fr::from(1u64), Fr::from(2u64)];
        let domain = Fr::from(3u64);
        let result = hash_with_domain(&inp, domain);
        let expected = Fr::hash_with_domain(inp, domain);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_msg() {
        let msgs = [
            &array::from_fn::<_, 1, _>(|i| Fr::from(i as u64))[..],
            &array::from_fn::<_, 10, _>(|i| Fr::from(i as u64))[..],
            &array::from_fn::<_, 11, _>(|i| Fr::from(i as u64))[..],
        ];

        let cap = [None, Some(1u128), Some(10), Some(11), Some(100)];

        for (msg, cap) in iproduct!(msgs.iter(), cap.iter()) {
            let result = hash_msg(msg, *cap);
            let expected = Fr::hash_msg(msg, *cap);
            assert_eq!(result, expected);
        }
    }

    fn hash_code_poseidon(code: &[u8]) -> [u8; 32] {
        let bytes_in_field = POSEIDON_HASH_BYTES_IN_FIELD;
        let fls = (0..(code.len() / bytes_in_field))
            .map(|i| i * bytes_in_field)
            .map(|i| {
                let mut buf: [u8; 32] = [0; 32];
                U256::from_big_endian(&code[i..i + bytes_in_field]).to_little_endian(&mut buf);
                Fr::from_bytes(&buf).unwrap()
            });
        let msgs: Vec<_> = fls
            .chain(if code.len() % bytes_in_field == 0 {
                None
            } else {
                let last_code = &code[code.len() - code.len() % bytes_in_field..];
                // pad to bytes_in_field
                let mut last_buf = vec![0u8; bytes_in_field];
                last_buf.as_mut_slice()[..last_code.len()].copy_from_slice(last_code);
                let mut buf: [u8; 32] = [0; 32];
                U256::from_big_endian(&last_buf).to_little_endian(&mut buf);
                Some(Fr::from_bytes(&buf).unwrap())
            })
            .collect();

        let h = if msgs.is_empty() {
            // the empty code hash is overlapped with simple hash on [0, 0]
            // an issue in poseidon primitive prevent us calculate it from hash_msg
            Fr::hash_with_domain([Fr::zero(), Fr::zero()], Fr::zero())
        } else {
            Fr::hash_msg(&msgs, Some(code.len() as u128 * HASHABLE_DOMAIN_SPEC))
        };
        let mut buf: [u8; 32] = [0; 32];
        U256::from_little_endian(h.to_repr().as_ref()).to_big_endian(&mut buf);
        buf
    }

    #[test]
    fn test_hash_code() {
        let codes = [
            &b""[..],
            &array::from_fn::<_, 1, _>(|i| i as u8)[..],
            &array::from_fn::<_, 32, _>(|i| i as u8)[..],
            &array::from_fn::<_, 33, _>(|i| i as u8)[..],
            &array::from_fn::<_, 64, _>(|i| i as u8)[..],
            &array::from_fn::<_, 65, _>(|i| i as u8)[..],
            &array::from_fn::<_, { 32 * 5 }, _>(|i| i as u8)[..],
            &array::from_fn::<_, { 32 * 5 + 16 }, _>(|i| i as u8)[..],
        ];

        for code in codes {
            let result = hash_code(code);
            let expected = hash_code_poseidon(code);
            assert_eq!(result, expected);
        }
    }
}
