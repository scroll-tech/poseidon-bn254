use sp1_prover::utils::get_cycles;
use sp1_sdk::SP1Stdin;

const ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-zkvm-elf");

fn main() {
    let cycles = get_cycles(ELF, &SP1Stdin::default());
    println!("final cycles: {cycles}");
}
