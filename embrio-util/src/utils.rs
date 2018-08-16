pub trait Captures<'a> {}

impl<T: ?Sized> Captures<'_> for T {}
