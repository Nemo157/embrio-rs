mod timer0;
mod timer1;

pub struct Timer<T>(T);

struct Interval<'a, T: 'a>(&'a mut T);
