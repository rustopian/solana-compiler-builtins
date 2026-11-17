# bench

Compute units of `memcpy`, `memmove`, `memset` and `memcmp` with the builtins that `cargo build-sbpf` links by default and with this branch's thresholds.

```sh
cargo install cargo-build-sbpf@0.3.1
cargo build-sbpf setup
rustup component add rust-src --toolchain nightly
bench/compare.sh
```

The script builds the program twice with plain `cargo build-sbpf` (default flags, no `.cargo/` configuration, no profile): once with the builtins it links by default, and once with `--features pr`, which links this branch's crate. Then it runs the Mollusk test in `tests/compare.rs`.

Each instruction does one operation:
- A fixed size is a constant in the program.
- A run-time size comes from the instruction data.
- The `helper::` rows call compare functions in `helper/`, a separate crate, with a constant length. The length becomes a constant only when the linker inlines them.

Each row is the compute units of the whole instruction; `no operation` is the harness alone. The `memcmp` buffers are equal, so each compare reads the full length.

Output with cargo-build-sbpf 0.3.1, sbpf-linker 0.2.8, nightly-2026-11-01 (rustc 1.101.0-nightly f9ceb4a4d, LLVM 23.1.4) and Mollusk 0.16.0. The default builtins are solana-compiler-builtins 0.3.0, the newest 0.3.x release on 2026-11-17. With another nightly, a row can change by a few units.

```
case                      default  this branch   diff
no operation                   24           24     +0
memset 40 B, fixed             29           29     +0
memset 48 B, fixed             94           45    -49
memset 64 B, fixed            112           45    -67
memset 80 B, fixed            130           45    -85
memset 81 B, fixed             46           45     -1
memcpy 0 B, run time           35           33     -2
memcpy 1 B, run time           44           36     -8
memcpy 2 B, run time           52           39    -13
memcpy 8 B, run time           47           44     -3
memcpy 16 B, run time          59           44    -15
memcpy 32 B, run time          83           44    -39
memcpy 64 B, run time          44           44     +0
memmove 0 B, run time          36           34     -2
memmove 1 B, run time          44           37     -7
memmove 2 B, run time          52           40    -12
memmove 8 B, run time          48           45     -3
memmove 16 B, run time         60           45    -15
memmove 32 B, run time         84           45    -39
memmove 64 B, run time         45           45     +0
memset 0 B, run time           35           33     -2
memset 1 B, run time           40           35     -5
memset 2 B, run time           45           37     -8
memset 8 B, run time           49           45     -4
memset 16 B, run time          58           45    -13
memset 32 B, run time          76           45    -31
memset 64 B, run time         112           45    -67
memcmp 0 B, run time           42           42     +0
memcmp 1 B, run time           52           52     +0
memcmp 2 B, run time           61           61     +0
memcmp 8 B, run time           52           52     +0
memcmp 16 B, run time          62           62     +0
memcmp 32 B, run time          82           82     +0
memcmp 64 B, run time          53           53     +0
helper::eq, 8 B                30           30     +0
helper::eq, 32 B               43           43     +0
helper::cmp, 8 B               29           29     +0
helper::cmp, 32 B              42           42     +0
```

- **Fixed `memset`.** `cargo build-sbpf` allows misaligned access and passes `--bpf-max-stores-per-memfunc=5`, so LLVM writes a fixed `memset` of up to 40 bytes as at most five 8-byte stores, and calls the builtin above that. From 41 to 80 bytes, 0.3.0 runs its loop and this branch calls `sol_memset_`. From 81 bytes, both call the syscall, and this branch's path is one jump shorter.
- **Run-time sizes** always reach the builtin, whatever the value of that flag: `memcpy`, `memmove` and `memset` as a call, and `memcmp` inlined by the linker. The `memmove` rows copy forward (the destination is below the source).
- **`memcmp`** keeps the 0.3.0 threshold of 32 bytes, so its rows do not change. With a threshold of 1 (`INLINE_MEMCMP_THRESHOLD` in `src/lib.rs`), a run-time `memcmp` costs up to 29 units less at these lengths and 1 more at 8 bytes, but the `helper::` rows cost 1 to 22 units more. The linker inlines the helper and `memcmp`, so the length becomes a constant. The 0.3.0 loop then folds into a few loads, while the syscall does not.
