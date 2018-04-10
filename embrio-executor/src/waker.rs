// TODO: Make this much much _MUCH_ safer to use

use core::ptr;
use core::cell::UnsafeCell;

use futures::{Async, Poll};
use futures::task::{UnsafeWake, Waker};
use cortex_m;
use cortex_m::interrupt::{self, Mutex};
use cortex_m::peripheral::NVIC;

pub struct EmbrioWaker {
    _reserved: u32,
}

struct EmbrioWakerInstance {
    instance: UnsafeCell<EmbrioWaker>,
}

struct SendEventWaker;

static EMBRIO_WAKER_INSTANCE: Mutex<EmbrioWakerInstance> = {
    Mutex::new(EmbrioWakerInstance {
        instance: UnsafeCell::new(EmbrioWaker { _reserved: 5 }),
    })
};

impl EmbrioWaker {
    pub(crate) fn waker() -> Waker {
        unsafe {
            Waker::new(EmbrioWaker::instance() as *mut UnsafeWake)
        }
    }

    pub(crate) fn instance() -> *mut EmbrioWaker {
        interrupt::free(|cs| {
            EMBRIO_WAKER_INSTANCE.borrow(cs).instance.get()
        })
    }

    // How this works.
    //
    // The NRF5x series of devices use an event system for all peripherals,
    // each peripheral is given an a block of 1024 32-bit words from address
    // 0x40000000 + 0x1000 * peripheral id, within this block at offset 0x100
    // are 32 32-bit registers reserved for events. Each event can have its
    // interrupt enabled or disabled by writing to the INTEN/INTENSET/INTENCLR
    // register for the peripheral, at a bit position corresponding to the
    // events register position. Not all peripherals have an INTEN register,
    // but they all (at least ones using events) appear to have both INTENSET
    // and INTENCLR registers at offsets of 0x304 and 0x308 respectively.
    //
    // By carefully checking and enabling/disabling these interrupts they can
    // be used as the wake system for the executor. The order of operations
    // here is very important, an overview of what happens to trigger an event
    // to occur and allow a state machine Future to step forward:
    //
    //  1. Something is started that will eventually cause an event to happen,
    //     e.g. for TIMER0 a value is placed in CC[0] and TASKS_START is
    //     triggered.
    //
    //  2. At some point in the future the thing that triggers the event
    //     happens, e.g. for TIMER0 the current counter matches what is in
    //     CC[0] and EVENTS_COMPARE[0] is triggered.
    //
    //  3. If the peripherals interrupt is enabled for the event that happened
    //     then the interrupt for that peripheral is marked as pending, e.g.
    //     for TIMER0 if INTEN[EVENTS_COMPARE[0]] == 1 then
    //     INTERRUPT_PENDING[TIMER0] <- 1
    //
    //  4. If the interrupt for that peripheral has just entered pending status
    //     then the Event Register will be set.
    //
    //  5. If the processer is waiting in an WFE instruction or attempts to run
    //     WFE then it will be woken if the Event Register is set and the Event
    //     Regiter will be cleared.
    //
    // This process runs in parallel to whatever the processor is currently
    // doing. Also, because we can have multiple peripherals/events being
    // waited on in a single task we could be woken up for EVENT_A, but then be
    // checking whether EVENT_B has occurred while EVENT_B is triggered. This
    // means that we have to be very careful about the order of checking the
    // event register, enabling/disabling interrupting for the event, clearing
    // the interrupt flag for the peripheral; to ensure that we don't
    // accidentally miss an event and get stuck sleeping until some other
    // unrelated event causes us to wake up.

    /// Will check whether an event has occurred, and if not automatically
    /// register interest in this event so the current task will be woken when
    /// it occurs.
    pub fn check(&mut self, peripheral: usize, event: usize) -> Poll<(), !> {
        assert!(peripheral < 32);
        assert!(event < 32);

        let peripheral_base = 0x40000000 + (0x1000 * peripheral);
        let intenset_register = peripheral_base + 0x304;
        let intenclr_register = peripheral_base + 0x308;
        let event_offset = 0x100 + (event * 4);
        let event_register = peripheral_base + event_offset;

        unsafe {
            let nvic = &*NVIC::ptr();
            if ptr::read_volatile(event_register as *mut u32) > 0 {
                // disable generating interrupt based on this event
                ptr::write_volatile(intenclr_register as *mut u32, 1 << event);
                // clear the interrupt pending flag
                nvic.icpr[0].write(1 << peripheral);
                Ok(Async::Ready(()))
            } else {
                // enable generating interrupt based on this event
                ptr::write_volatile(intenset_register as *mut u32, 1 << event);
                Ok(Async::Pending)
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn check_and_clear(&mut self, peripheral: usize, event: usize) -> Poll<(), !> {
        assert!(peripheral < 32);
        assert!(event < 32);

        let peripheral_base = 0x40000000 + (0x1000 * peripheral);
        let event_offset = 0x100 + (event * 4);
        let event_register = peripheral_base + event_offset;

        try_ready!(self.check(peripheral, event));
        unsafe { ptr::write_volatile(event_register as *mut u32, 0) };
        Ok(Async::Ready(()))
    }

    pub(crate) fn wait() {
        cortex_m::asm::wfe();
    }
}

impl SendEventWaker {
    fn waker() -> Waker {
        static INSTANCE: SendEventWaker = SendEventWaker;
        unsafe {
            Waker::new(&INSTANCE as *const SendEventWaker as *mut SendEventWaker as *mut UnsafeWake)
        }
    }
}

unsafe impl UnsafeWake for EmbrioWaker {
    unsafe fn clone_raw(&self) -> Waker {
        SendEventWaker::waker()
    }

    unsafe fn drop_raw(&self) {
        panic!("Should never be dropped since we clone to a different type");
    }

    unsafe fn wake(&self) {
        panic!("Need to use SendEventWaker to do in software wakes");
    }
}

unsafe impl UnsafeWake for SendEventWaker {
    unsafe fn clone_raw(&self) -> Waker {
        SendEventWaker::waker()
    }

    unsafe fn drop_raw(&self) {
        // No-op, singleton
    }

    unsafe fn wake(&self) {
        unimplemented!("https://github.com/japaric/cortex-m/pull/86");
    }
}
