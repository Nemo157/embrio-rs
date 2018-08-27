use core::{cmp, mem::PinMut, cell::RefCell, marker::PhantomData};
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
pub struct Uart<'b> {
    _marker: PhantomData<(
        &'b mut UART0,
        &'b mut NVIC,
    )>,
}

#[derive(Debug)]
pub struct Tx<'a, 'b: 'a> {
    _marker: PhantomData<(
        &'a mut Uart<'b>,
        &'a mut Pin<'b, Output<PushPull>>,
    )>,
}

#[derive(Debug)]
pub struct Rx<'a, 'b: 'a> {
    _marker: PhantomData<(
        &'a mut Uart<'b>,
        &'a mut Pin<'b, Input<Floating>>,
    )>,
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
    uart: &'static mut UART0,
    nvic: &'static mut NVIC,
    events: Events,
    rx_waker: Option<Waker>,
    tx: TxContext,
}

static CONTEXT: Mutex<RefCell<Option<Context>>> = Mutex::new(RefCell::new(None));

unsafe fn erase_lifetime<'a, T>(t: &'a mut T) -> &'static mut T {
    &mut *(t as *mut T)
}

impl<'b> Uart<'b> {
    pub(crate) fn new(
        uart: &'b mut UART0,
        nvic: &'b mut NVIC,
    ) -> Self {
        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            assert!(context.is_none());
            context.replace(Context {
                uart: unsafe { erase_lifetime(uart) },
                nvic: unsafe { erase_lifetime(nvic) },
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
            });
        });

        Uart { _marker: PhantomData }
    }

    pub fn init<'a>(
        &'a mut self,
        txpin: &'a mut Pin<'b, Output<PushPull>>,
        rxpin: &'a mut Pin<'b, Input<Floating>>,
        speed: BAUDRATEW,
    ) -> (Tx<'a, 'b>, Rx<'a, 'b>) where 'b: 'a {
        free(|c| {
            let mut context = CONTEXT.borrow(c).borrow_mut();
            let context = context.as_mut().unwrap();

            context.uart.txd.write(|w| unsafe { w.bits(0) });
            context.uart.pseltxd
                .write(|w| unsafe { w.bits(txpin.get_id() as u32) });
            context.uart.pselrxd
                .write(|w| unsafe { w.bits(rxpin.get_id() as u32) });
            context.uart.baudrate.write(|w| w.baudrate().variant(speed));
            context.uart.intenset.write(|w| w.rxdrdy().set().txdrdy().set());
            context.uart.enable.write(|w| w.enable().enabled());

            context.uart.tasks_starttx.write(|w| unsafe { w.bits(1) });
            context.uart.tasks_startrx.write(|w| unsafe { w.bits(1) });

            context.nvic.enable(Interrupt::UART0);
        });

        (Tx { _marker: PhantomData }, Rx { _marker: PhantomData })
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

impl<'b> Drop for Uart<'b> {
    fn drop(&mut self) {
        free(|c| {
            let context = CONTEXT.borrow(c).borrow_mut().take().unwrap();

            context.nvic.disable(Interrupt::UART0);

            context.uart.tasks_stoptx.write(|w| unsafe { w.bits(1) });
            context.uart.tasks_stoprx.write(|w| unsafe { w.bits(1) });

            context.uart.enable.write(|w| w.enable().disabled());
            context.uart.intenclr.write(|w| w.rxdrdy().clear().txdrdy().clear());

            context.uart.pseltxd.reset();
            context.uart.pselrxd.reset();
            context.uart.baudrate.reset();
        });
    }
}

impl<'a, 'b: 'a> io::Read for Rx<'a, 'b> {
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

impl<'a, 'b: 'a> io::Write for Tx<'a, 'b> {
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
