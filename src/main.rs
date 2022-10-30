#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use stm32f0xx_hal as hal;
use core::fmt::Write;

use cortex_m_rt::entry;
use hal::pac;
use hal::prelude::*;
use hal::serial::Serial;
use hal::spi::{Spi, Mode, Phase, Polarity};
use nrf24_rs::config::{NrfConfig, PALevel, PayloadSize, DataPipe};
use nrf24_rs::Nrf24l01;

#[entry]
fn main() -> ! {
    if let Some(p) = pac::Peripherals::take() {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

        let gpioa = p.GPIOA.split(&mut rcc);
        let gpiob = p.GPIOB.split(&mut rcc);
        let gpioc = p.GPIOC.split(&mut rcc);

        let (tx, rx, sclk, miso, mosi, ncs, ce) = cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa2.into_alternate_af1(cs),
                gpioa.pa3.into_alternate_af1(cs),
                gpioa.pa5.into_alternate_af0(cs),
                gpioa.pa6.into_alternate_af0(cs),
                gpioa.pa7.into_alternate_af0(cs),
                gpiob.pb6.into_push_pull_output(cs),
                gpioc.pc7.into_push_pull_output(cs),
            )
        });

        let mut serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

        let spi = Spi::spi1(p.SPI1, (sclk, miso, mosi), Mode {
            polarity: Polarity::IdleHigh,
            phase: Phase::CaptureOnSecondTransition,
        }, 4.mhz(), &mut rcc);

        let cp = cortex_m::Peripherals::take().unwrap();
        let mut delay = hal::delay::Delay::new(cp.SYST, &rcc);

        let config = NrfConfig::default()
            .channel(8)
            .pa_level(PALevel::Min)
            .payload_size(PayloadSize::Dynamic);

        let mut nrf = Nrf24l01::new(spi, ce, ncs, &mut delay, config).unwrap();
        if !nrf.is_connected().unwrap() {
            panic!("Chip is not connected.");
        }

        nrf.open_writing_pipe(b"1SRVR").unwrap();

        let message: [u8; 8] = [0, 1, b'1', b'C', b'L', b'N', b'T', 0];

        // Keep trying to send the message
        while let Err(e) = nrf.write(&mut delay, &message) {
            write!(serial, "Error while sending message: {:?}\r\n", e).unwrap();
            // Something went wrong while writing, try again in 50ms
            delay.delay_ms(50u16);
        }

        let mut buf: [u8; 32] = [0; 32];

        nrf.open_reading_pipe(DataPipe::DP0, b"1CLNT").unwrap();
        nrf.start_listening().unwrap();

        write!(serial, "Hello\r\n").unwrap();

        while let Ok(true) = nrf.data_available() {
            match nrf.read(&mut buf) {
                Err(e) => write!(serial, "Error while reading data from buffer: {:?}\r\n", e).unwrap(),
                Ok(n) => {
                    write!(serial, "Successfully read {} bytes of data!\r\n", n).unwrap();
                    write!(serial, "Received value: {:?}\r\n", buf).unwrap();
                },
            };
        }
    }

    loop {
        continue;
    }
}
