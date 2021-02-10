#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use nb;
use stm32f0xx_hal::{pac, prelude::*, serial::Serial};

#[entry]
fn main() -> ! {
    if let Some(p) = pac::Peripherals::take() {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

        let gpioa = p.GPIOA.split(&mut rcc);

        let (tx, rx) = cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa2.into_alternate_af1(cs),
                gpioa.pa3.into_alternate_af1(cs),
            )
        });

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
