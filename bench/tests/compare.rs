use mollusk_svm::Mollusk;
use solana_instruction::Instruction;

const PROGRAM_ID: [u8; 32] = [7; 32];
const FIXED: [(u8, &str); 6] = [
    (0, "no operation"),
    (1, "memset 40 B, fixed"),
    (2, "memset 48 B, fixed"),
    (3, "memset 64 B, fixed"),
    (4, "memset 80 B, fixed"),
    (5, "memset 81 B, fixed"),
];
const RUNTIME: [(u8, &str); 4] = [(6, "memcpy"), (7, "memmove"), (8, "memset"), (9, "memcmp")];
const LENGTHS: [u8; 7] = [0, 1, 2, 8, 16, 32, 64];
const HELPER: [(u8, &str); 4] = [
    (10, "helper::eq, 8 B"),
    (11, "helper::eq, 32 B"),
    (12, "helper::cmp, 8 B"),
    (13, "helper::cmp, 32 B"),
];

fn load(name: &str) -> Mollusk {
    let path = format!("{}/target/{name}", env!("CARGO_MANIFEST_DIR"));
    Mollusk::new(&PROGRAM_ID.into(), &path)
}

fn cu(mollusk: &Mollusk, case: u8, n: u8) -> u64 {
    let mut data = vec![5u8; 264];
    data[..2].copy_from_slice(&[case, n]);
    let instruction = Instruction::new_with_bytes(PROGRAM_ID.into(), &data, vec![]);
    let result = mollusk.process_instruction(&instruction, &[]);
    assert!(
        result.program_result.is_ok(),
        "case {case}, n {n}: {:?}",
        result.program_result
    );
    result.compute_units_consumed
}

#[test]
fn compare() {
    let default = load("bench-default");
    let pr = load("bench-pr");
    println!(
        "\n{:<24} {:>8} {:>12} {:>6}",
        "case", "default", "this branch", "diff"
    );
    let row = |label: &str, case: u8, n: u8| {
        let (a, b) = (cu(&default, case, n), cu(&pr, case, n));
        println!("{label:<24} {a:>8} {b:>12} {:>+6}", b as i64 - a as i64);
    };
    for (case, label) in FIXED {
        row(label, case, 0);
    }
    for (case, name) in RUNTIME {
        for n in LENGTHS {
            row(&format!("{name} {n} B, run time"), case, n);
        }
    }
    for (case, label) in HELPER {
        row(label, case, 0);
    }
}
