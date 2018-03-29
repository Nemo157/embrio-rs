use futures::{task, Async, Future, FutureExt, Poll, Stream, StreamExt};
use nrf51::TIMER0;

use embrio::si::{Time, time::microsecond};

pub struct Timer(TIMER0);

struct Interval<'a>(&'a mut TIMER0);

impl Timer {
    pub fn new(timer: TIMER0) -> Timer {
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

impl Timer {
    pub fn timeout(
        &mut self,
        duration: Time,
    ) -> impl Future<Item = (), Error = !> + '_ {
        self.interval(duration)
            .next()
            .map(|(r, _)| r.unwrap())
            .map_err(|(e, _)| e)
    }

    pub fn interval(
        &mut self,
        duration: Time,
    ) -> impl Stream<Item = (), Error = !> + '_ {
        let us = duration.get(microsecond);
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0
            .tasks_clear
            .write(|w| unsafe { w.bits(1) });
        self.0
            .tasks_start
            .write(|w| unsafe { w.bits(1) });

        Interval(&mut self.0)
    }
}

impl<'a> Stream for Interval<'a> {
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
