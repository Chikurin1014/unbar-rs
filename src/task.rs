use display_interface_spi::SPIInterface;
use embassy_time::Timer;
use embedded_hal_bus::{spi::AtomicDevice, util::AtomicCell};
use esp_hal::{
    gpio,
    ledc::{self, HighSpeed},
    spi, Async,
};
use libm::*;
use log::*;

use crate::{
    controll::system::{System, SystemIFace as _},
    hw::Hardware,
};

#[embassy_executor::task]
pub async fn controll(
    mut hardware: Hardware<'static>,
    ledc_timer: ledc::timer::Timer<'static, HighSpeed>,
    mut system: System,
) {
    hardware.attach_timer(&ledc_timer).unwrap();
    loop {
        let delay_timer = Timer::after_millis(10);

        let res = hardware.imu.bno055.accel_data();
        let input;
        if res.is_ok() {
            input = res.unwrap();
        } else {
            error!("IMU error: {:?}", res.unwrap_err());
            continue;
        }
        if input.x == -0.01 || input.y == -0.01 || input.z == -0.01 {
            warn!("IMU returned invalid value");
            continue;
        }

        let output = system.step(&());
        hardware.set_motor_speed(&output).unwrap();
        info!("ax:    {:e}", input.x);
        info!("ay:    {:e}", input.y);
        info!("az:    {:e}", input.z);
        info!("theta: {:e}", atan2f(input.y, input.z));

        delay_timer.await;
    }
}

#[embassy_executor::task]
pub async fn display(
    spi: AtomicCell<spi::master::Spi<'static, Async>>,
    display_cs: gpio::Output<'static>,
    display_dc: gpio::Output<'static>,
    display_reset: gpio::Output<'static>,
    mut delay: embassy_time::Delay,
) {
    let mut _display = ili9341::Ili9341::new(
        SPIInterface::new(
            AtomicDevice::new(&spi, display_cs, delay.clone()).unwrap(),
            display_dc,
        ),
        display_reset,
        &mut delay,
        ili9341::Orientation::Portrait,
        ili9341::DisplaySize320x480,
    )
    .unwrap();
    loop {
        info!("Display task");
        Timer::after_millis(100).await;
    }
}
