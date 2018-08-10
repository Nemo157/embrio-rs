use core::{mem::PinMut, time::Duration};
use futures_core::{task, Future, Poll, Stream};
use nrf51::TIMER0;

use super::{Interval, Timeout, Timer};

impl Timer<TIMER0> {
    pub fn timer0(timer: TIMER0) -> Timer<TIMER0> {
        // 32bits @ 1MHz == max delay of ~1 hour 11 minutes
        timer.bitmode.write(|w| w.bitmode()._32bit());
        timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
        timer.shorts.write(|w| w.compare0_clear().enabled());

        Timer(timer)
    }
}

impl<'a> embrio_core::timer::Timer for &'a mut Timer<TIMER0> {
    type Error = !;

    type Timeout = Timeout<'a, TIMER0>;

    type Interval = Interval<'a, TIMER0>;

    fn timeout(self, duration: Duration) -> Self::Timeout {
        let us = (duration.as_secs() as u32 * 1000) + duration.subsec_millis();
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        Timeout(Some(self))
    }

    fn interval(self, duration: Duration) -> Self::Interval {
        let us = (duration.as_secs() as u32 * 1000) + duration.subsec_millis();
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        Interval(self)
    }
}

impl<'a> Future for Timeout<'a, TIMER0> {
    type Output = Result<&'a mut Timer<TIMER0>, !>;

    fn poll(
        mut self: PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Self::Output> {
        self.0
            .as_mut()
            .unwrap()
            .0
            .intenclr
            .write(|w| w.compare0().clear());
        if self.0.as_mut().unwrap().0.events_compare[0].read().bits() == 1 {
            self.0.as_mut().unwrap().0.events_compare[0].reset();
            Poll::Ready(Ok(self.0.take().unwrap()))
        } else {
            self.0
                .as_mut()
                .unwrap()
                .0
                .intenset
                .write(|w| w.compare0().set());
            Poll::Pending
        }
    }
}

impl<'a> Stream for Interval<'a, TIMER0> {
    type Item = Result<(), !>;

    fn poll_next(
        self: PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Option<Self::Item>> {
        (self.0).0.intenclr.write(|w| w.compare0().clear());
        if (self.0).0.events_compare[0].read().bits() == 1 {
            (self.0).0.events_compare[0].reset();
            Poll::Ready(Some(Ok(())))
        } else {
            (self.0).0.intenset.write(|w| w.compare0().set());
            Poll::Pending
        }
    }
}
