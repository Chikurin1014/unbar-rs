#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_hal_bus::{spi::AtomicDevice, util::AtomicCell};
use esp_backtrace as _;
use esp_hal::{
    gpio, i2c,
    ledc::{self, channel::ChannelIFace as _, timer::TimerIFace as _},
    spi,
    time::RateExtU32 as _,
    timer::timg,
};
use libm::*;
use log::*;

use unbar_rs::{
    controll::{self, system::SystemIFace as _},
    hw,
};

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = {
        let cfg = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
        esp_hal::init(cfg)
    };
    esp_alloc::heap_allocator!(72 * 1024);
    esp_println::logger::init_logger_from_env();

    // setup embassy
    let embassy_timg = timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(embassy_timg.timer0);

    // setup delay
    let delay = embassy_time::Delay;

    // setup ledc
    let mut ledc = ledc::Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(ledc::LSGlobalClkSource::APBClk);
    let mut timer = ledc.timer::<ledc::HighSpeed>(ledc::timer::Number::Timer0);
    timer
        .configure(ledc::timer::config::Config {
            duty: ledc::timer::config::Duty::Duty10Bit,
            clock_source: ledc::timer::HSClockSource::APBClk,
            frequency: 1.kHz(),
        })
        .unwrap();

    // setup spi
    let spi = AtomicCell::new(
        spi::master::Spi::new(peripherals.SPI2, spi::master::Config::default())
            .unwrap()
            .into_async(),
    );

    let mut _hw = hw::Hardware::new(
        gpio::Output::new(peripherals.GPIO4, gpio::Level::Low),
        gpio::Output::new(peripherals.GPIO16, gpio::Level::Low),
        {
            let mut ch =
                ledc::channel::Channel::new(ledc::channel::Number::Channel0, peripherals.GPIO17);
            ch.configure(ledc::channel::config::Config {
                timer: &timer,
                duty_pct: 0,
                pin_config: ledc::channel::config::PinConfig::PushPull,
            })
            .unwrap();
            ch
        },
        gpio::Output::new(peripherals.GPIO14, gpio::Level::Low),
        gpio::Output::new(peripherals.GPIO33, gpio::Level::Low),
        {
            let mut ch =
                ledc::channel::Channel::new(ledc::channel::Number::Channel1, peripherals.GPIO32);
            ch.configure(ledc::channel::config::Config {
                timer: &timer,
                duty_pct: 0,
                pin_config: ledc::channel::config::PinConfig::PushPull,
            })
            .unwrap();
            ch
        },
        i2c::master::I2c::new(
            peripherals.I2C0,
            i2c::master::Config::default()
                .with_frequency(50.kHz())
                .with_timeout(i2c::master::BusTimeout::Maximum),
        )
        .unwrap()
        .with_scl(peripherals.GPIO22)
        .with_sda(peripherals.GPIO21)
        .into_async(),
        gpio::Output::new(peripherals.GPIO27, gpio::Level::High),
        display_interface_spi::SPIInterface::new(
            AtomicDevice::new(
                &spi,
                gpio::Output::new(peripherals.GPIO5, gpio::Level::Low),
                delay.clone(),
            )
            .unwrap(),
            gpio::Output::new(peripherals.GPIO2, gpio::Level::Low),
        ),
        gpio::Output::new(peripherals.GPIO15, gpio::Level::Low),
        delay,
    )
    .await;

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {}

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
