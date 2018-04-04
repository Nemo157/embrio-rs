mod timer0;
mod timer1;

pub struct Timer<T>(T);

pub struct Timeout<'a, T: 'a>(Option<&'a mut Timer<T>>);
pub struct Interval<'a, T: 'a>(&'a mut Timer<T>);
