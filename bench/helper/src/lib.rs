//! Compares in a separate crate that take the length at run time. When the program passes a constant,
//! the length is known only after the linker inlines these functions.
#![no_std]

pub fn eq(a: &[u8], b: &[u8], n: usize) -> bool {
    if a.len() < n || b.len() < n {
        return false;
    }
    a[..n] == b[..n]
}

pub fn cmp(a: &[u8], b: &[u8], n: usize) -> i8 {
    if a.len() < n || b.len() < n {
        return 2;
    }
    a[..n].cmp(&b[..n]) as i8
}
