#![no_std]
#![no_main]
#![allow(unused_imports)]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_time::Timer;
use log::info;
use panic_probe as _;
use cyw43_pio::PioSpi;
use embassy_rp::peripherals::USB;

// Bind interrupts to their handlers.
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

// Async task for USB logging.
#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize peripherals and USB driver.
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    // Start Wi-Fi task
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    //Buttons
    let share = Input::new(p.PIN_9, Pull::Up);
    let options = Input::new(p.PIN_10, Pull::Up);
    let xbox = Input::new(p.PIN_16, Pull::Up);
    let lb = Input::new(p.PIN_11, Pull::Up);
    let lt = Input::new(p.PIN_14, Pull::Up);
    let rb = Input::new(p.PIN_12, Pull::Up);
    let rt = Input::new(p.PIN_15, Pull::Up);

    // Spawn the logger task
    spawner.spawn(logger_task(usb_driver)).unwrap();
    
    Timer::after_millis(1000).await;
    info!("Hello, world!");

    loop {
        
        //TODO: Send through UDP information about the buttons
        
        Timer::after_millis(10).await;
        //info!("Hello, world!");
    }
}
