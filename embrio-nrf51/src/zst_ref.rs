use core::{any::type_name, fmt, marker::PhantomData, mem, ops::Deref};

pub struct ZstRef<'a, T: 'a> {
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> ZstRef<'a, T> {
    /// Create a new [`ZstRef`] to a value
    pub fn new(value: &'a T) -> Self {
        let _value = value;
        // TODO: could this be a static assert?
        assert!(mem::size_of::<T>() == 0);
        ZstRef {
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Deref for ZstRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(0xDEAD_BEEF as *const T) }
    }
}

impl<'a, T: 'a> Clone for ZstRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: 'a> Copy for ZstRef<'a, T> {}

impl<'a, T: 'a> fmt::Debug for ZstRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("ZstRef<")?;
        f.write_str(type_name::<T>())?;
        f.write_str(">")?;
        Ok(())
    }
}
