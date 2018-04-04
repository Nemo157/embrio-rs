use futures::{task, Async, Future, Poll, Stream};
use nrf51::TIMER0;

use embrio;
use embrio::si::{Time, time::microsecond};

use super::{Interval, Timeout, Timer};

impl Timer<TIMER0> {
    pub fn new(timer: TIMER0) -> Timer<TIMER0> {
        // 32bits @ 1MHz == max delay of ~1 hour 11 minutes
        timer.bitmode.write(|w| w.bitmode()._32bit());
        timer
            .prescaler
            .write(|w| unsafe { w.prescaler().bits(4) });
        timer
            .shorts
            .write(|w| w.compare0_clear().enabled());

        Timer(timer)
    }
}

impl embrio::timer::Timer for Timer<TIMER0> {
    type Error = !;

    type Timeout = Timeout<TIMER0>;

    type Interval = Interval<TIMER0>;

    fn timeout(self, duration: Time) -> Self::Timeout {
        let us = duration.get(microsecond);
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0
            .tasks_clear
            .write(|w| unsafe { w.bits(1) });
        self.0
            .tasks_start
            .write(|w| unsafe { w.bits(1) });

        Timeout(Some(self.0))
    }

    fn interval(self, duration: Time) -> Self::Interval {
        let us = duration.get(microsecond);
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0
            .tasks_clear
            .write(|w| unsafe { w.bits(1) });
        self.0
            .tasks_start
            .write(|w| unsafe { w.bits(1) });

        Interval(self.0)
    }
}

impl Future for Timeout<TIMER0> {
    type Item = Timer<TIMER0>;
    type Error = !;

    fn poll(
        &mut self,
        _cx: &mut task::Context,
    ) -> Poll<Self::Item, Self::Error> {
        self.0
            .as_mut()
            .unwrap()
            .intenclr
            .write(|w| w.compare0().clear());
        if self.0.as_mut().unwrap().events_compare[0]
            .read()
            .bits() == 1
        {
            self.0.as_mut().unwrap().events_compare[0].reset();
            Ok(Async::Ready(Timer(self.0.take().unwrap())))
        } else {
            self.0
                .as_mut()
                .unwrap()
                .intenset
                .write(|w| w.compare0().set());
            Ok(Async::Pending)
        }
    }
}

impl Stream for Interval<TIMER0> {
    type Item = ();
    type Error = !;

    fn poll_next(
        &mut self,
        _cx: &mut task::Context,
    ) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.intenclr.write(|w| w.compare0().clear());
        if self.0.events_compare[0].read().bits() == 1 {
            self.0.events_compare[0].reset();
            Ok(Async::Ready(Some(())))
        } else {
            self.0.intenset.write(|w| w.compare0().set());
            Ok(Async::Pending)
        }
    }
}
