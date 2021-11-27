use std::{
    alloc::{self, Layout},
    mem, ptr,
};

use crate::limits;

pub(crate) fn into_inner<T, Container>(ptr: *mut T, container: Container) -> T {
    let t = unsafe { ptr::read(ptr) };

    // need to free the box without running T's dtor
    if !limits::should_inline::<T>() {
        unsafe {
            alloc::dealloc(ptr as *mut u8, Layout::new::<T>());
        }
        mem::forget(container);
    }

    t
}
