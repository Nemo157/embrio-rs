use core::{cmp, mem::PinMut, cell::RefCell};
use futures_core::task::{self, Poll, Waker};
use embrio_core::io;
use cortex_m::{peripheral::NVIC, interrupt::{free, Mutex}};
use nrf51::{UART0, Interrupt};

use crate::{
    gpio::{
        mode::{Floating, Input, Output, PushPull},
        Pin,
    },
};

pub use nrf51::uart0::baudrate::BAUDRATEW;

#[derive(Debug)]
pub struct Uart<'a> {
    _txpin: &'a mut Pin<'a, Output<PushPull>>,
    _rxpin: &'a mut Pin<'a, Input<Floating>>,
}

#[derive(Debug)]
pub struct Tx<'a> {
    _txpin: &'a mut Pin<'a, Output<PushPull>>,
}

#[derive(Debug)]
pub struct Rx<'a> {
    _rxpin: &'a mut Pin<'a, Input<Floating>>,
}

struct Events {
    rxdrdy: bool,
    txdrdy: bool,
}

struct TxContext {
    waker: Option<Waker>,
    buffer: [u8; 8],
    to_send: u8,
    sent: u8,
}

struct Context {
    uart: UART0,
    nvic: NVIC,
    events: Events,
    rx_waker: Option<Waker>,
    tx: TxContext,
}

static CONTEXT: Mutex<RefCell<Option<Context>>> = Mutex::new(RefCell::new(None));

impl<'a> Uart<'a> {
    pub fn new(
        uart: UART0,
        txpin: &'a mut Pin<'a, Output<PushPull>>,
        rxpin: &'a mut Pin<'a, Input<Floating>>,
        speed: BAUDRATEW,
        mut nvic: NVIC,
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
        free(|c| {
            CONTEXT.borrow(c).replace(Some(Context {
                uart,
                nvic,
                events: Events {
                    rxdrdy: false,
                    txdrdy: false,
                },
                rx_waker: None,
                tx: TxContext {
                    waker: None,
                    buffer: [0; 8],
                    to_send: 0,
                    sent: 0,
                }
            }));
        });

        Uart { _txpin: txpin, _rxpin: rxpin }
    }

    pub fn split(self) -> (Tx<'a>, Rx<'a>) {
        let Uart { _txpin, _rxpin } = self;
        (Tx { _txpin }, Rx { _rxpin })
    }

    #[doc(hidden)]
    pub fn interrupt() {
        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            let context = context.as_mut().unwrap();
            context.nvic.clear_pending(Interrupt::UART0);
            if context.uart.events_rxdrdy.read().bits() == 1 {
                context.uart.events_rxdrdy.reset();
                context.events.rxdrdy = true;
                if let Some(waker) = context.rx_waker.as_ref() {
                    waker.wake();
                }
            }
            if context.uart.events_txdrdy.read().bits() == 1 {
                context.uart.events_txdrdy.reset();
                if context.tx.sent < context.tx.to_send {
                    let byte = context.tx.buffer[context.tx.sent as usize];
                    context.tx.sent += 1;
                    context.uart.txd.write(|w| unsafe { w.bits(byte.into()) });
                } else {
                    context.events.txdrdy = true;
                    if let Some(waker) = context.tx.waker.as_ref() {
                        waker.wake();
                    }
                }
            }
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

        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            let context = context.as_mut().unwrap();
            if context.events.rxdrdy {
                context.events.rxdrdy = false;
                buf[0] = context.uart.rxd.read().bits() as u8;
                context.rx_waker = None;
                Poll::Ready(Ok(1))
            } else {
                context.rx_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        })
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

        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            let context = context.as_mut().unwrap();
            if context.events.txdrdy {
                let length = cmp::min(buf.len() - 1, context.tx.buffer.len());
                context.events.txdrdy = false;
                context.tx.to_send = length as u8;
                context.tx.sent = 0;
                context.tx.buffer[..length].copy_from_slice(&buf[1..length + 1]);
                context.tx.waker = None;
                context.uart.txd.write(|w| unsafe { w.bits(buf[0].into()) });
                Poll::Ready(Ok(length + 1))
            } else {
                context.tx.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        })
    }

    fn poll_flush(
        self: PinMut<'_, Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            let context = context.as_mut().unwrap();
            if context.events.txdrdy {
                // We don't reset the event here because it's used to keep track of
                // whether there is an outstanding write or not.
                context.tx.waker = None;
                Poll::Ready(Ok(()))
            } else {
                context.tx.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        })
    }

    fn poll_close(
        self: PinMut<'_, Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            let context = context.as_mut().unwrap();
            context.tx.waker = None;
            context.uart.tasks_stoptx.write(|w| unsafe { w.bits(1) });
            Poll::Ready(Ok(()))
        })
    }
}
