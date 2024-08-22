#![no_main]

sp1_zkvm::entrypoint!(main);

use poseidon_bn254::{hash_with_domain, Fr};

fn main() {
    let inp = [Fr::from(1u64), Fr::from(2u64)];
    let domain = Fr::from(3u64);
    let result = hash_with_domain(inp, domain);
}
