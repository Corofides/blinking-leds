#![no_std]
#![no_main]

use panic_halt as _;
use avr_device::atmega328p;

#[avr_device::entry]
fn main() -> ! {
    let mut number: i32 = 0;

    let mut is_on = false;

    let dp = atmega328p::Peripherals::take().unwrap();

    // let timer: *mut u8 = 0x46 as *mut u8;

    let mut last_timer_value: u8 = 0;
    let mut total_timer_value: u32 = 0;

    let cycles_per_second: u32 =  (16000000.0 / 1024.0) as u32;

    dp.TC0.tccr0b.write(|w| {
        w.cs0().prescale_1024()
    });

    dp.PORTB.ddrb.write(|w| w.pb5().set_bit());
    //dp.PORTB.portb.write(|w| w.pb5().set_bit());

    loop {
        
        let current_timer = dp.TC0.tcnt0.read().bits();
        let delta: u32;

        if current_timer < last_timer_value {
            let last_timer_value: u32 = last_timer_value as u32;
            let mut current_timer: u32 = current_timer as u32;
            
            current_timer = current_timer + 256;

            delta = current_timer - last_timer_value

        } else {
            delta = (current_timer - last_timer_value) as u32;
        }

        total_timer_value = total_timer_value.wrapping_add(delta);
        last_timer_value = current_timer;

        number = number.wrapping_add(1);
        
        if total_timer_value > cycles_per_second {

            dp.PORTB.portb.write(|w| w.pb5().set_bit());

            if is_on {
                dp.PORTB.portb.write(|w| w.pb5().clear_bit());
            } else {
                dp.PORTB.portb.write(|w| w.pb5().set_bit());
            }

            total_timer_value = total_timer_value.saturating_sub(cycles_per_second);
            is_on = !is_on;
        }
        
    }
}
