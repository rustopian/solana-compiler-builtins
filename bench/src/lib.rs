//! One memory operation per instruction. Instruction data: `[case, n, ...]`, where bytes 8..136
//! and 136..264 are the source buffers.
#![cfg_attr(target_arch = "bpf", no_std)]

#[cfg(all(target_arch = "bpf", feature = "pr"))]
use solana_compiler_builtins as _;

#[cfg(target_arch = "bpf")]
mod program {
    use core::{hint::black_box, mem::MaybeUninit, ptr, slice};

    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo) -> ! {
        loop {}
    }

    type Case = unsafe fn(*const u8, *const u8, *mut u8, usize);

    #[inline(never)]
    unsafe fn noop(_: *const u8, _: *const u8, _: *mut u8, _: usize) {}

    macro_rules! fixed_memset {
        ($name:ident, $n:literal) => {
            #[inline(never)]
            unsafe fn $name(_: *const u8, _: *const u8, dst: *mut u8, _: usize) {
                unsafe { ptr::write_bytes(dst, 0, $n) }
            }
        };
    }

    fixed_memset!(memset_40, 40);
    fixed_memset!(memset_48, 48);
    fixed_memset!(memset_64, 64);
    fixed_memset!(memset_80, 80);
    fixed_memset!(memset_81, 81);

    #[inline(never)]
    unsafe fn memcpy_n(a: *const u8, _: *const u8, dst: *mut u8, n: usize) {
        unsafe { slice::from_raw_parts_mut(dst, n).copy_from_slice(slice::from_raw_parts(a, n)) }
    }

    #[inline(never)]
    unsafe fn memmove_n(a: *const u8, _: *const u8, dst: *mut u8, n: usize) {
        unsafe { ptr::copy(a, dst, n) }
    }

    #[inline(never)]
    unsafe fn memset_n(_: *const u8, _: *const u8, dst: *mut u8, n: usize) {
        unsafe { slice::from_raw_parts_mut(dst, n).fill(0) }
    }

    #[inline(never)]
    unsafe fn memcmp_n(a: *const u8, b: *const u8, dst: *mut u8, n: usize) {
        let order = unsafe { slice::from_raw_parts(a, n).cmp(slice::from_raw_parts(b, n)) };
        unsafe { *dst = order as u8 }
    }

    macro_rules! helper_compare {
        ($name:ident, $f:ident, $n:literal) => {
            #[inline(never)]
            unsafe fn $name(a: *const u8, b: *const u8, dst: *mut u8, _: usize) {
                let (a, b) =
                    unsafe { (slice::from_raw_parts(a, 128), slice::from_raw_parts(b, 128)) };
                unsafe { *dst = helper::$f(a, b, $n) as u8 }
            }
        };
    }

    helper_compare!(helper_eq_8, eq, 8);
    helper_compare!(helper_eq_32, eq, 32);
    helper_compare!(helper_cmp_8, cmp, 8);
    helper_compare!(helper_cmp_32, cmp, 32);

    const CASES: [Case; 14] = [
        noop,
        memset_40,
        memset_48,
        memset_64,
        memset_80,
        memset_81,
        memcpy_n,
        memmove_n,
        memset_n,
        memcmp_n,
        helper_eq_8,
        helper_eq_32,
        helper_cmp_8,
        helper_cmp_32,
    ];

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
        unsafe {
            let data = input.add(16);
            let n = *data.add(1) as usize;
            let Some(case) = CASES.get(*data as usize) else {
                return 1;
            };
            if n > 128 {
                return 2;
            }
            let mut buf = MaybeUninit::<[u8; 128]>::uninit();
            black_box(*case)(data.add(8), data.add(136), buf.as_mut_ptr().cast(), n);
        }
        0
    }
}
