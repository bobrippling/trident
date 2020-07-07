use std::marker::PhantomData;
use std::mem;
use std::ptr;

const NWORDS: usize = 3;

/**
 * A struct that stores a `T`, either inline or, if `T` is larger than 3 words, allocated.
 */
#[repr(C)]
pub struct Trident<T> {
    _align: [T; 0],
    words: [usize; NWORDS],
    _phantom: PhantomData<T>,
}

const LIMIT: usize = mem::size_of::<[usize; NWORDS]>();

impl<T> Trident<T> {
    fn should_inline() -> bool {
        mem::size_of::<T>() <= LIMIT
    }

    pub fn new(t: T) -> Self {
        if Self::should_inline() {
            let mut ret = Self {
                words: [0; NWORDS],
                _phantom: PhantomData {},
                _align: Default::default(),
            };

            unsafe {
                ptr::copy_nonoverlapping(&t, ret.as_mut_ref(), 1);
            }
            mem::forget(t);

            ret
        } else {
            let alloc = Box::new(t);

            Self {
                words: [Box::into_raw(alloc) as usize, 0, 0],
                _phantom: PhantomData {},
                _align: Default::default(),
            }
        }
    }

    pub fn as_ref(&self) -> &T {
        let ptr = if Self::should_inline() {
            &self.words as *const _ as usize as *const T
        } else {
            self.words[0] as *const T
        };

        unsafe { &*ptr }
    }

    pub fn as_mut_ref(&mut self) -> &mut T {
        let ptr = if Self::should_inline() {
            &mut self.words as *mut _ as usize as *mut T
        } else {
            self.words[0] as *mut T
        };

        unsafe { &mut *ptr }
    }
}

impl<T> Trident<T>
where
    T: Copy,
{
    pub fn get(&self) -> T {
        *self.as_ref()
    }
}

impl<T> Drop for Trident<T> {
    fn drop(&mut self) {
        let ptr = self.as_mut_ref();

        if Self::should_inline() {
            unsafe {
                ptr::drop_in_place(ptr);
            }
        } else {
            unsafe {
                Box::from_raw(ptr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Trident;

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct SmallCopy {
        i: i32,
        j: u32,
    }
    impl Copy for SmallCopy {}

    #[derive(PartialEq, Eq, Debug)]
    struct Large([i32; 20]);

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct LargeCopy([i32; 20]);
    impl Copy for LargeCopy {}

    /// Small Types

    #[test]
    fn handles_small_type() {
        assert!(Trident::<i32>::should_inline());

        let t = Trident::new(3);

        assert_eq!(t.as_ref(), &3);
    }

    #[test]
    fn handles_small_copy_type() {
        assert!(Trident::<SmallCopy>::should_inline());

        let t = Trident::new(SmallCopy { i: 1, j: 2 });

        assert_eq!(t.get(), SmallCopy { i: 1, j: 2 });
    }

    /// Large Types

    #[test]
    fn handles_large_type() {
        assert!(!Trident::<Large>::should_inline());

        let mut large1 = Large(Default::default());
        let mut large2 = Large(Default::default());

        for (i, (p1, p2)) in large1.0.iter_mut().zip(large2.0.iter_mut()).enumerate() {
            *p1 = i as i32;
            *p2 = i as i32;
        }

        let large1 = large1;
        let large2 = large2;

        let t = Trident::new(large1);

        assert_eq!(*t.as_ref(), large2);
    }

    #[test]
    fn handles_large_copy_type() {
        assert!(!Trident::<LargeCopy>::should_inline());

        let mut large = LargeCopy(Default::default());

        for (i, p) in large.0.iter_mut().enumerate() {
            *p = i as i32;
        }

        let t = Trident::new(large);

        assert_eq!(t.get(), large);
    }

    /// Drop Implementation

    #[test]
    fn handles_small_dtor_type() {
        assert!(Trident::<Dtor>::should_inline());

        struct Dtor<'a> {
            x: i32,
            y: i32,
            drops: &'a mut u32,
        }

        impl Drop for Dtor<'_> {
            fn drop(&mut self) {
                assert_eq!(self.x, 8217);
                assert_eq!(self.y, 924);

                *self.drops += 1;
            }
        }

        let mut drops = 0;
        let dtor = Dtor {
            drops: &mut drops,
            x: 8217,
            y: 924,
        };

        let t = Trident::new(dtor);

        drop(t);

        assert_eq!(drops, 1);
    }

    #[test]
    fn handles_large_dtor_type() {
        assert!(!Trident::<Dtor>::should_inline());

        struct Dtor<'a> {
            ents: [usize; 12],
            drops: &'a mut u32,
        }

        impl Drop for Dtor<'_> {
            fn drop(&mut self) {
                for (i, &ent) in self.ents.iter().enumerate() {
                    assert_eq!(i, ent);
                }

                *self.drops += 1;
            }
        }

        let mut drops = 0;
        let mut dtor = Dtor {
            ents: Default::default(),
            drops: &mut drops,
        };
        for (i, p) in dtor.ents.iter_mut().enumerate() {
            *p = i;
        }
        let dtor = dtor;

        let t = Trident::new(dtor);

        drop(t);

        assert_eq!(drops, 1);
    }
}
