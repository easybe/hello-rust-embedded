#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use nb;
use cortex_m_rt::entry;
use stm32f0xx_hal as hal;
use hal::pac;
use hal::prelude::*;
use hal::serial::Serial;
use hal::stm32f0::stm32f0x1::{interrupt, Interrupt, NVIC};
use hal::time::Hertz;
use hal::timers::{Timer, Event};

static mut LED : Option<hal::gpio::gpioa::PA5<hal::gpio::Output<
    hal::gpio::PushPull>>> = None;

#[interrupt]
fn TIM2() {
    unsafe {
        (*pac::TIM2::ptr()).sr.modify(|_, w| w.uif().clear());

        let led = LED.as_mut().unwrap();
        let _ = led.toggle();
    }
}

#[entry]
fn main() -> ! {
    if let Some(p) = pac::Peripherals::take() {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

        let gpioa = p.GPIOA.split(&mut rcc);

        let (tx, rx, led) = cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa2.into_alternate_af1(cs),
                gpioa.pa3.into_alternate_af1(cs),
                gpioa.pa5.into_push_pull_output(cs)
            )
        });

        unsafe {
            LED = Some(led);
        }

        unsafe {
            NVIC::unmask(Interrupt::TIM2);
        }
        let mut timer = Timer::tim2(p.TIM2, Hertz(1), &mut rcc);
        timer.listen(Event::TimeOut);

        let mut serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

        for byte in b"Hello, world!\r\n" {
            nb::block!(serial.write(*byte)).unwrap();
        }

        loop {
            let received = nb::block!(serial.read()).unwrap();
            nb::block!(serial.write(received)).ok();
        }
    }

    loop {
        continue;
    }
}
