use core::mem;

use embrio;
use futures::{task, Async, Poll};
use nrf51::UART0;

use gpio::{Pin, mode::{Floating, Input, Output, PushPull}};
use zst_ref::ZstRef;

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
        uart.baudrate
            .write(|w| w.baudrate().variant(speed));
        uart.enable.write(|w| w.enable().enabled());

        uart.tasks_starttx
            .write(|w| unsafe { w.bits(1) });
        uart.tasks_startrx
            .write(|w| unsafe { w.bits(1) });

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

impl<'a> embrio::io::Read for Rx<'a> {
    type Error = !;

    fn poll_read(
        self: mem::Pin<Self>,
        _cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(Async::Ready(0));
        }

        self.uart.intenclr.write(|w| w.rxdrdy().clear());
        if self.uart.events_rxdrdy.read().bits() == 1 {
            self.uart.events_rxdrdy.reset();
            buf[0] = self.uart.rxd.read().bits() as u8;
            Ok(Async::Ready(1))
        } else {
            self.uart.intenset.write(|w| w.rxdrdy().set());
            Ok(Async::Pending)
        }
    }
}

impl<'a> embrio::io::Write for Tx<'a> {
    type Error = !;

    fn poll_write(
        self: mem::Pin<Self>,
        _cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(Async::Ready(0));
        }

        self.uart.intenclr.write(|w| w.txdrdy().clear());
        if self.uart.events_txdrdy.read().bits() == 1 {
            self.uart.events_txdrdy.reset();
            self.uart
                .txd
                .write(|w| unsafe { w.bits(buf[0].into()) });
            Ok(Async::Ready(1))
        } else {
            self.uart.intenset.write(|w| w.txdrdy().set());
            Ok(Async::Pending)
        }
    }

    fn poll_flush(
        self: mem::Pin<Self>,
        _cx: &mut task::Context,
    ) -> Poll<(), Self::Error> {
        self.uart.intenclr.write(|w| w.txdrdy().clear());
        if self.uart.events_txdrdy.read().bits() == 1 {
            Ok(Async::Ready(()))
        } else {
            self.uart.intenset.write(|w| w.txdrdy().set());
            Ok(Async::Pending)
        }
    }

    fn poll_close(
        self: mem::Pin<Self>,
        _cx: &mut task::Context,
    ) -> Poll<(), Self::Error> {
        self.uart
            .tasks_stoptx
            .write(|w| unsafe { w.bits(1) });
        Ok(Async::Ready(()))
    }
}
