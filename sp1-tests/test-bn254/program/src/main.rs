#![no_main]

sp1_zkvm::entrypoint!(main);

use itertools::iproduct;
use poseidon_bn254::{hash_code, hash_msg, hash_with_domain, Fr};
use std::array;

fn main() {
    println!("cycle-tracker-start: hash_with_domain(&[Fr::zero(), Fr::zero()], Fr::zero())");
    hash_with_domain(&[Fr::zero(), Fr::zero()], Fr::zero());
    println!("cycle-tracker-end: hash_with_domain(&[Fr::zero(), Fr::zero()], Fr::zero())");

    println!(
        "cycle-tracker-start: hash_with_domain(&[Fr::from(1u64), Fr::from(2u64)], Fr::from(3u64))"
    );
    hash_with_domain(&[Fr::from(1u64), Fr::from(2u64)], Fr::from(3u64));
    println!(
        "cycle-tracker-end: hash_with_domain(&[Fr::from(1u64), Fr::from(2u64)], Fr::from(3u64))"
    );

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
        let result = hash_code(code);
        println!("{}", format!("cycle-tracker-end: {tag}"));
    }
}
