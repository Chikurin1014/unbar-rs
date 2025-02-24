use display_interface_spi::SPIInterface;
use embassy_time::Delay;
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::{
    gpio::Output,
    i2c,
    ledc::{self, HighSpeed},
    spi, Async,
};

use ili9341::Ili9341;

pub mod component;

use component::{Imu, Motor};

pub struct Hardware<'a> {
    pub left_motor: Motor<'a>,
    pub right_motor: Motor<'a>,
    pub imu: Imu<'a>,
    pub display: Ili9341<
        SPIInterface<AtomicDevice<'a, spi::master::Spi<'a, Async>, Output<'a>, Delay>, Output<'a>>,
        Output<'a>,
    >,
}

impl<'a> Hardware<'a> {
    pub async fn new(
        left_motor_dir1: Output<'a>,
        left_motor_dir2: Output<'a>,
        left_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
        right_motor_dir1: Output<'a>,
        right_motor_dir2: Output<'a>,
        right_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
        i2c: i2c::master::I2c<'a, Async>,
        imu_reset: Output<'a>,
        spi_interface: SPIInterface<
            AtomicDevice<'a, spi::master::Spi<'a, Async>, Output<'a>, Delay>,
            Output<'a>,
        >,
        display_reset: Output<'a>,
        mut delay: Delay,
    ) -> Self {
        Self {
            left_motor: Motor::new(left_motor_dir1, left_motor_dir2, left_motor_pwm_ch),
            right_motor: Motor::new(right_motor_dir1, right_motor_dir2, right_motor_pwm_ch),
            imu: Imu::new(i2c, imu_reset, &mut delay).await,
            display: Ili9341::new(
                spi_interface,
                display_reset,
                &mut delay,
                ili9341::Orientation::Portrait,
                ili9341::DisplaySize240x320,
            )
            .unwrap(),
        }
    }

    pub fn set_motor_speed(
        &mut self,
        speed: &crate::controll::output::MotorSpeed,
    ) -> Result<(), ledc::channel::Error> {
        self.left_motor.set_speed(speed.left)?;
        self.right_motor.set_speed(speed.right)
    }
}
