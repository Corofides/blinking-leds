#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
use avr_device::atmega328p::{self, PORTB};
use avr_device::interrupt::{self, Mutex};
use core::cell::RefCell;

static OVERFLOW_COUNT: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));
static LED_13: Mutex<RefCell<Option<LED>>> = Mutex::new(RefCell::new(None));

#[avr_device::interrupt(atmega328p)]
fn TIMER0_OVF() {
    let cycles_per_second: u8 = 61; // 16Mhz / (1024 * 256) Rounded down as it's closer.
   
    let overflow_count = interrupt::free(|cs| {
        let mut overflow_count = OVERFLOW_COUNT.borrow(cs).borrow_mut();
        *overflow_count = overflow_count.wrapping_add(1);
        *overflow_count
    });

    if overflow_count >= cycles_per_second {

        interrupt::free(|cs| {

            OVERFLOW_COUNT.borrow(cs).replace(0);

            let mut led_option = LED_13.borrow(cs).borrow_mut();

            if let Some(led) = led_option.as_mut() {
                led.toggle_led();
            }
        });

    }
    
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

    let dp = atmega328p::Peripherals::take().unwrap();

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

    unsafe {
        avr_device::interrupt::enable();
    }

    // Enable interrupts.
    loop { /* Do Nothing */ }
}
