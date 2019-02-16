use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Poll, Waker},
    time::Duration,
};

use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use futures_core::stream::Stream;
use nrf51::{Interrupt, TIMER1};

use super::{Interval, Timeout, Timer};

static TIMER1_WAKER: Mutex<RefCell<Option<Waker>>> =
    Mutex::new(RefCell::new(None));

impl Timer<TIMER1> {
    pub fn timer1(timer: TIMER1, nvic: &mut NVIC) -> Timer<TIMER1> {
        // 32bits @ 1MHz == max delay of ~1 hour 11 minutes
        timer.bitmode.write(|w| w.bitmode()._32bit());
        timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
        timer.shorts.write(|w| w.compare0_clear().enabled());

        nvic.enable(Interrupt::TIMER1);

        Timer(timer)
    }

    fn register_waker(waker: Waker) {
        free(|c| {
            TIMER1_WAKER.borrow(c).replace(Some(waker));
        });
    }

    #[doc(hidden)]
    pub fn interrupt() {
        free(|c| {
            if let Some(waker) = &*TIMER1_WAKER.borrow(c).borrow() {
                waker.wake();
            }
        });
    }
}

impl<'a> embrio_core::timer::Timer for &'a mut Timer<TIMER1> {
    type Error = !;

    type Timeout = Timeout<'a, TIMER1>;

    type Interval = Interval<'a, TIMER1>;

    fn timeout(self, duration: Duration) -> Self::Timeout {
        let us = (duration.as_secs() as u32 * 1000) + duration.subsec_millis();
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.intenset.write(|w| w.compare0().set());

        self.0.events_compare[0].reset();
        self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        Timeout(Some(self))
    }

    fn interval(self, duration: Duration) -> Self::Interval {
        let us = (duration.as_secs() as u32 * 1000) + duration.subsec_millis();
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.intenset.write(|w| w.compare0().set());

        self.0.events_compare[0].reset();
        self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        Interval(self)
    }
}

impl<'a> Future for Timeout<'a, TIMER1> {
    type Output = Result<&'a mut Timer<TIMER1>, !>;

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        Timer::<TIMER1>::register_waker(waker.clone());
        if self.0.as_mut().unwrap().0.events_compare[0].read().bits() == 1 {
            self.0.as_mut().unwrap().0.events_compare[0].reset();
            Poll::Ready(Ok(self.0.take().unwrap()))
        } else {
            Poll::Pending
        }
    }
}

impl<'a> Stream for Interval<'a, TIMER1> {
    type Item = Result<(), !>;

    fn poll_next(
        self: Pin<&mut Self>,
        waker: &Waker,
    ) -> Poll<Option<Self::Item>> {
        Timer::<TIMER1>::register_waker(waker.clone());
        if (self.0).0.events_compare[0].read().bits() == 1 {
            (self.0).0.events_compare[0].reset();
            Poll::Ready(Some(Ok(())))
        } else {
            Poll::Pending
        }
    }
}
