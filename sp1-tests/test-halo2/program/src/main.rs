#![no_main]

sp1_zkvm::entrypoint!(main);

use itertools::iproduct;
use poseidon_base::hash::{Hashable, MessageHashable};
use poseidon_bn254::Fr;
use std::array;
use ethers_core::k256::elliptic_curve::PrimeField;
use ethers_core::types::U256;

fn main() {
    Fr::hash_with_domain([Fr::zero(), Fr::zero()], Fr::zero()); // init poseidon base

    println!("cycle-tracker-start: hash_with_domain(&[Fr::zero(), Fr::zero()], Fr::zero())");
    hash_with_domain(&[Fr::zero(), Fr::zero()], Fr::zero());
    println!("cycle-tracker-end: hash_with_domain(&[Fr::zero(), Fr::zero()], Fr::zero())");

    println!("cycle-tracker-start: hash_with_domain(&[Fr::from(1u64), Fr::from(2u64)], Fr::from(3u64))");
    hash_with_domain(&[Fr::from(1u64), Fr::from(2u64)], Fr::from(3u64));
    println!("cycle-tracker-end: hash_with_domain(&[Fr::from(1u64), Fr::from(2u64)], Fr::from(3u64))");

    let msgs = [
        &array::from_fn::<_, 1, _>(|i| Fr::from(i as u64))[..],
        &array::from_fn::<_, 10, _>(|i| Fr::from(i as u64))[..],
        &array::from_fn::<_, 11, _>(|i| Fr::from(i as u64))[..],
    ];

    let cap = [None, Some(100)];

    for (msg, cap) in iproduct!(msgs.iter(), cap.iter()) {
        let tag = format!("hash_msg({}, {:?})", msg.len(), cap);
        println!("{}", format!("cycle-tracker-start: {tag}"));
        hash_msg(msg, *cap);
        println!("{}", format!("cycle-tracker-end: {tag}"));
    }

    let codes = [
        &[],
        &array::from_fn::<_, 1, _>(|i| i as u8)[..],
        &array::from_fn::<_, 128, _>(|i| i as u8)[..],
        &array::from_fn::<_, 256, _>(|i| i as u8)[..],
        &array::from_fn::<_, 512, _>(|i| i as u8)[..],
        &array::from_fn::<_, 1024, _>(|i| i as u8)[..],
        &array::from_fn::<_, 2048, _>(|i| i as u8)[..],
        &array::from_fn::<_, 4096, _>(|i| i as u8)[..],
        &array::from_fn::<_, 16384, _>(|i| i as u8)[..],
        &array::from_fn::<_, 24576, _>(|i| i as u8)[..],
    ];

    for code in codes.iter() {
        let tag = format!("hash_code({})", code.len());
        println!("{}", format!("cycle-tracker-start: {tag}"));
        println!("cycle-tracker-start: poseidon_bn254");
        let result = poseidon_bn254::hash_code(code);
        println!("cycle-tracker-end: poseidon_bn254");

        println!("cycle-tracker-start: poseidon_base");
        let expected = hash_code_poseidon(code);
        println!("cycle-tracker-end: poseidon_base");
        assert_eq!(result, expected);
        println!("{}", format!("cycle-tracker-end: {tag}"));
    }

}

fn hash_with_domain(inp: &[Fr; 2], domain: Fr) {
    println!("cycle-tracker-start: poseidon_bn254");
    let result = poseidon_bn254::hash_with_domain(inp, domain);
    println!("cycle-tracker-end: poseidon_bn254");

    let inp = *inp;
    println!("cycle-tracker-start: poseidon_base");
    let expected = Fr::hash_with_domain(inp, domain);
    println!("cycle-tracker-end: poseidon_base");
    assert_eq!(result, expected);
}

fn hash_msg(msg: &[Fr], cap: Option<u128>) {
    println!("cycle-tracker-start: poseidon_bn254");
    let result = poseidon_bn254::hash_msg(msg, cap);
    println!("cycle-tracker-end: poseidon_bn254");

    println!("cycle-tracker-start: poseidon_base");
    let expected = Fr::hash_msg(msg, cap);
    println!("cycle-tracker-end: poseidon_base");

    assert_eq!(result, expected);
}

fn hash_code_poseidon(code: &[u8]) -> [u8; 32] {
    const HASHABLE_DOMAIN_SPEC: u128 = 1 << 64;
    let bytes_in_field = 31;
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