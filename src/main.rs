#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
use avr_device::atmega328p::{self, PORTB};
use avr_device::interrupt::{self, Mutex};
use core::cell::RefCell;

static TIMER_OVERFLOWED: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static LED_13: Mutex<RefCell<Option<LED>>> = Mutex::new(RefCell::new(None));

#[avr_device::interrupt(atmega328p)]
fn TIMER0_OVF() {
    interrupt::free(|cs| {
        TIMER_OVERFLOWED.borrow(cs).replace(true);
    });
}

pub struct LED {
    pub port: PORTB,
}

impl LED {
    pub fn toggle_led(&self) {
        self.port.pinb.write(|w| w.pb5().set_bit());
    }
}

#[avr_device::entry]
fn main() -> ! {

    // Enable interrupts.
    unsafe {
        avr_device::interrupt::enable();
    }
    let mut overflow_count: u32 = 0;

    let dp = atmega328p::Peripherals::take().unwrap();

    let cycles_per_second: u32 =  (16000000.0 / 1024.0) as u32;

    // Set up the prescaler.
    dp.TC0.tccr0b.write(|w| {
        w.cs0().prescale_1024()
    });

    // Enable the overflow interrupt.
    dp.TC0.timsk0.write(|w| {
        w.toie0().set_bit()
    });

    // Allow us to use GPIO13 as output.
    dp.PORTB.ddrb.write(|w| w.pb5().set_bit());

    let led13 = LED {
        port: dp.PORTB,
    };

    interrupt::free(move |cs| { 
        LED_13.borrow(cs).replace(Some(led13));
    });

    loop {

        // Check if we've overflowed.
        let timer_overflowed = interrupt::free(|cs| {
            let mut value = TIMER_OVERFLOWED.borrow(cs).borrow_mut();

            if *value {
                *value = false;
                true
            } else {
                false
            }

        });

        // If we've overflowed add 1 to our time count.
        if timer_overflowed {
            overflow_count = overflow_count.wrapping_add(1);
        }

        // Each overflow counts as 256 ticks.
        if (overflow_count * 256) >= cycles_per_second {
            overflow_count = 0;

            interrupt::free(|cs| {

                let mut led_option = LED_13.borrow(cs).borrow_mut();

                // .as_mut is important as we don't want to take ownership of the
                // value, just get a reference to it.
                if let Some(led) = led_option.as_mut() {
                    led.toggle_led();
                }

            });

        }
                
    }
}
