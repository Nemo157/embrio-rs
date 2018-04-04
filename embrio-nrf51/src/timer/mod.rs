mod timer0;
mod timer1;

pub struct Timer<T>(T);

pub struct Timeout<T>(Option<T>);
pub struct Interval<T>(T);
