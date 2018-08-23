use core::{mem::PinMut, cell::RefCell};
use futures_core::task::{self, Poll, Waker};
use embrio_core::io;
use cortex_m::{peripheral::NVIC, interrupt::{free, Mutex}};
use nrf51::{UART0, Interrupt};

use crate::{
    gpio::{
        mode::{Floating, Input, Output, PushPull},
        Pin,
    },
    zst_ref::ZstRef,
};

pub use nrf51::uart0::baudrate::BAUDRATEW;

#[derive(Debug)]
pub struct Uart<'a> {
    uart: ZstRef<'a, UART0>,
    _txpin: &'a mut Pin<'a, Output<PushPull>>,
    _rxpin: &'a mut Pin<'a, Input<Floating>>,
}

#[derive(Debug)]
pub struct Tx<'a> {
    uart: ZstRef<'a, UART0>,
    _txpin: &'a mut Pin<'a, Output<PushPull>>,
}

#[derive(Debug)]
pub struct Rx<'a> {
    uart: ZstRef<'a, UART0>,
    _rxpin: &'a mut Pin<'a, Input<Floating>>,
}

static UART0_RX_WAKER: Mutex<RefCell<Option<Waker>>> = Mutex::new(RefCell::new(None));
static UART0_TX_WAKER: Mutex<RefCell<Option<Waker>>> = Mutex::new(RefCell::new(None));

impl<'a> Uart<'a> {
    pub fn new(
        uart: &'a mut UART0,
        txpin: &'a mut Pin<'a, Output<PushPull>>,
        rxpin: &'a mut Pin<'a, Input<Floating>>,
        speed: BAUDRATEW,
        nvic: &mut NVIC,
    ) -> Self {
        uart.txd.write(|w| unsafe { w.bits(0) });
        uart.pseltxd
            .write(|w| unsafe { w.bits(txpin.get_id() as u32) });
        uart.pselrxd
            .write(|w| unsafe { w.bits(rxpin.get_id() as u32) });
        uart.baudrate.write(|w| w.baudrate().variant(speed));
        uart.intenset.write(|w| w.rxdrdy().set().txdrdy().set());
        uart.enable.write(|w| w.enable().enabled());

        uart.tasks_starttx.write(|w| unsafe { w.bits(1) });
        uart.tasks_startrx.write(|w| unsafe { w.bits(1) });

        nvic.enable(Interrupt::UART0);

        let uart = ZstRef::new(uart);
        Uart {
            uart,
            _txpin: txpin,
            _rxpin: rxpin,
        }
    }

    pub fn split(self) -> (Tx<'a>, Rx<'a>) {
        let Uart {
            uart,
            _txpin,
            _rxpin,
        } = self;
        (Tx { uart, _txpin }, Rx { uart, _rxpin })
    }

    #[doc(hidden)]
    pub fn interrupt() {
        free(|c| {
            if let Some(waker) = &*UART0_RX_WAKER.borrow(c).borrow() {
                waker.wake();
            }
            if let Some(waker) = &*UART0_TX_WAKER.borrow(c).borrow() {
                waker.wake();
            }
        });
    }
}

impl<'a> Rx<'a> {
    fn register_waker(waker: Waker) {
        free(|c| {
            UART0_RX_WAKER.borrow(c).replace(Some(waker));
        });
    }
}

impl<'a> Tx<'a> {
    fn register_waker(waker: Waker) {
        free(|c| {
            UART0_TX_WAKER.borrow(c).replace(Some(waker));
        });
    }
}

impl<'a> io::Read for Rx<'a> {
    type Error = !;

    fn poll_read(
        self: PinMut<'_, Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        Self::register_waker(cx.waker().clone());
        if self.uart.events_rxdrdy.read().bits() == 1 {
            self.uart.events_rxdrdy.reset();
            buf[0] = self.uart.rxd.read().bits() as u8;
            Poll::Ready(Ok(1))
        } else {
            Poll::Pending
        }
    }
}

impl<'a> io::Write for Tx<'a> {
    type Error = !;

    fn poll_write(
        self: PinMut<'_, Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        Self::register_waker(cx.waker().clone());
        if self.uart.events_txdrdy.read().bits() == 1 {
            self.uart.events_txdrdy.reset();
            self.uart.txd.write(|w| unsafe { w.bits(buf[0].into()) });
            Poll::Ready(Ok(1))
        } else {
            Poll::Pending
        }
    }

    fn poll_flush(
        self: PinMut<'_, Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Self::register_waker(cx.waker().clone());
        if self.uart.events_txdrdy.read().bits() == 1 {
            // We don't reset the event here because it's used to keep track of
            // whether there is an outstanding write or not.
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn poll_close(
        self: PinMut<'_, Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        self.uart.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        Poll::Ready(Ok(()))
    }
}
