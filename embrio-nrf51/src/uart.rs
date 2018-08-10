use core::mem;

use embrio_core::io;
use embrio_executor::EmbrioContext;
use futures_core::{task, Poll};
use futures_util::ready;
use nrf51::UART0;

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

impl<'a> Uart<'a> {
    pub fn new(
        uart: &'a UART0,
        txpin: &'a mut Pin<'a, Output<PushPull>>,
        rxpin: &'a mut Pin<'a, Input<Floating>>,
        speed: BAUDRATEW,
    ) -> Self {
        uart.txd.write(|w| unsafe { w.bits(0) });
        uart.pseltxd
            .write(|w| unsafe { w.bits(txpin.get_id() as u32) });
        uart.pselrxd
            .write(|w| unsafe { w.bits(rxpin.get_id() as u32) });
        uart.baudrate.write(|w| w.baudrate().variant(speed));
        uart.enable.write(|w| w.enable().enabled());

        uart.tasks_starttx.write(|w| unsafe { w.bits(1) });
        uart.tasks_startrx.write(|w| unsafe { w.bits(1) });

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
}

impl<'a> io::Read for Rx<'a> {
    type Error = !;

    fn poll_read(
        self: mem::PinMut<Self>,
        _cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        if self.uart.events_rxdrdy.read().bits() == 1 {
            self.uart.events_rxdrdy.reset();
            self.uart.intenclr.write(|w| w.rxdrdy().clear());
            buf[0] = self.uart.rxd.read().bits() as u8;
            Poll::Ready(Ok(1))
        } else {
            self.uart.intenset.write(|w| w.rxdrdy().set());
            Poll::Pending
        }
    }
}

impl<'a> io::Write for Tx<'a> {
    type Error = !;

    #[allow(unreachable_code)]
    fn poll_write(
        self: mem::PinMut<Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        ready!(cx.waker().check_and_clear(2, 7));
        self.uart.txd.write(|w| unsafe { w.bits(buf[0].into()) });
        Poll::Ready(Ok(1))
    }

    fn poll_flush(
        self: mem::PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        self.uart.intenset.write(|w| w.txdrdy().set());
        if self.uart.events_txdrdy.read().bits() == 1 {
            self.uart.intenclr.write(|w| w.txdrdy().clear());
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn poll_close(
        self: mem::PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        self.uart.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        Poll::Ready(Ok(()))
    }
}
