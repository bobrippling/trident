use std::mem;

pub(crate) const NWORDS: usize = 3;

pub(crate) const SIZE_LIMIT: usize = mem::size_of::<[usize; NWORDS]>();

pub(crate) fn should_inline<T>() -> bool {
    mem::size_of::<T>() <= SIZE_LIMIT
}
