/**
 * A struct that stores a type-erased `T`, either inline or, if `T` is larger than 3 words, allocated.
 */
use std::{mem, ptr};

use crate::into;
use crate::limits::{self, NWORDS};
use crate::Trident;

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
compile_error!("Not a 32- or 64-bit machine");

#[cfg_attr(target_pointer_width = "64", repr(C, align(8)))]
#[cfg_attr(target_pointer_width = "32", repr(C, align(4)))]
pub struct Erased {
    words: [usize; NWORDS],
}

impl Erased {
    /**
     * Create an `Erased` from a `T`
     *
     * `T`'s destructor cannot be run, as the type is erased.
     */
    pub fn new<T>(t: T) -> Self {
        if limits::should_inline::<T>() {
            let mut ret = Self { words: [0; NWORDS] };

            unsafe {
                ptr::copy_nonoverlapping(&t, ret.as_mut_ref(), 1);
            }
            mem::forget(t);

            ret
        } else {
            let alloc = Box::new(t);

            Self {
                words: [Box::into_raw(alloc) as usize, 0, 0],
            }
        }
    }

    /**
     * Get a pointer to the contained `T`.
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn as_ptr<T>(&self) -> *const T {
        if limits::should_inline::<T>() {
            &self.words as *const _ as usize as *const T
        } else {
            self.words[0] as *const T
        }
    }

    /**
     * Get a reference to the contained `T`.
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn as_ref<T>(&self) -> &T {
        &*self.as_ptr()
    }

    /**
     * Get a mutable pointer to the contained `T`.
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn as_mut_ptr<T>(&mut self) -> *mut T {
        if limits::should_inline::<T>() {
            &mut self.words as *mut _ as usize as *mut T
        } else {
            self.words[0] as *mut T
        }
    }

    /**
     * Get a mutable reference to the contained `T`.
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn as_mut_ref<T>(&mut self) -> &mut T {
        &mut *self.as_mut_ptr()
    }

    /**
     * Copy the contained `T`.
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn get<T: Copy>(&self) -> T {
        *self.as_ref()
    }

    /**
     * Get the contained `T`.
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn into_inner<T>(mut self) -> T {
        into::into_inner(self.as_mut_ptr(), self)
    }

    /**
     * Convert to a `Trident<T>`
     * Unsafe because we don't know that this is the same `T` that this `Erased` was created with.
     */
    pub unsafe fn into_trident<T>(self) -> Trident<T> {
        Trident::from_erased(self)
    }
}
