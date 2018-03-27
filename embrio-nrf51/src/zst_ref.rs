use core::marker::PhantomData;
use core::mem;
use core::ops::Deref;

#[derive(Clone, Copy, Debug)]
pub struct ZstRef<'a, T: 'a> {
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> ZstRef<'a, T> {
    /// Create a new [`ZstRef`] to a value
    pub fn new(value: &'a T) -> Self {
        let _value = value;
        // TODO: could this be a static assert?
        assert!(mem::size_of::<T>() == 0);
        ZstRef { marker: PhantomData }
    }
}

impl<'a, T: 'a> Deref for ZstRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(0xDEADBEEF as *const T) }
    }
}
