#![allow(clippy::needless_range_loop)]
#![allow(clippy::op_ref)]
#![allow(unexpected_cfgs)]

mod constants;
mod imp;

#[cfg(feature = "bn254")]
pub use bn254::Fr;
#[cfg(feature = "halo2curves_v1")]
pub use halo2curves_v1::bn256::Fr;
#[cfg(feature = "halo2curves_v3")]
pub use halo2curves_v3::bn256::Fr;

pub use constants::*;

pub type State = [Fr; T];
pub type Mds = [[Fr; T]; T];

pub fn hash_with_domain(inp: [Fr; 2], domain: Fr) -> Fr {
    let mut state = [domain, inp[0], inp[1]];
    imp::permute(&mut state);
    state[0]
}

#[cfg(all(test, feature = "halo2curves_v1"))]
mod tests {
    use super::*;
    use poseidon_base::Hashable;

    #[test]
    fn test_hash() {
        let inp = [Fr::from(1u64), Fr::from(2u64)];
        let domain = Fr::from(3u64);
        let result = hash_with_domain(inp, domain);
        let expected = Fr::hash_with_domain(inp, domain);
        assert_eq!(result, expected);
    }
}
