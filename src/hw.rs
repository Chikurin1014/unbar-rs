use embassy_time::Delay;
use esp_hal::{
    gpio::Output,
    i2c,
    ledc::{self, HighSpeed},
    Async,
};

pub mod component;

use component::{Imu, Motor};

pub struct Hardware<'a> {
    pub left_motor: Motor<'a, HighSpeed>,
    pub right_motor: Motor<'a, HighSpeed>,
    pub imu: Imu<'a>,
}

pub struct HardwareBuilder<'a> {
    left_motor_dir1: Output<'a>,
    left_motor_dir2: Output<'a>,
    left_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
    right_motor_dir1: Output<'a>,
    right_motor_dir2: Output<'a>,
    right_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
    i2c: i2c::master::I2c<'a, Async>,
    imu_reset: Output<'a>,
    delay: Delay,
}

impl<'a> HardwareBuilder<'a> {
    pub async fn build(self) -> Hardware<'a> {
        Hardware {
            left_motor: Motor::new(
                self.left_motor_dir1,
                self.left_motor_dir2,
                self.left_motor_pwm_ch,
            ),
            right_motor: Motor::new(
                self.right_motor_dir1,
                self.right_motor_dir2,
                self.right_motor_pwm_ch,
            ),
            imu: Imu::new(self.i2c, self.imu_reset, &mut self.delay.clone()).await,
        }
    }
}

impl<'a> Hardware<'a> {
    pub fn builder(
        left_motor_dir1: Output<'a>,
        left_motor_dir2: Output<'a>,
        left_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
        right_motor_dir1: Output<'a>,
        right_motor_dir2: Output<'a>,
        right_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
        i2c: i2c::master::I2c<'a, Async>,
        imu_reset: Output<'a>,
        delay: Delay,
    ) -> HardwareBuilder<'a> {
        HardwareBuilder {
            left_motor_dir1,
            left_motor_dir2,
            left_motor_pwm_ch,
            right_motor_dir1,
            right_motor_dir2,
            right_motor_pwm_ch,
            i2c,
            imu_reset,
            delay: delay,
        }
    }

    pub fn attach_timer(
        &mut self,
        timer: &'a ledc::timer::Timer<'a, HighSpeed>,
    ) -> Result<(), ledc::channel::Error> {
        self.left_motor.attach_timer(timer)?;
        self.right_motor.attach_timer(timer)
    }

    pub fn set_motor_speed(
        &mut self,
        speed: &crate::controll::output::MotorSpeed,
    ) -> Result<(), ledc::channel::Error> {
        self.left_motor.set_speed(speed.left)?;
        self.right_motor.set_speed(speed.right)
    }
}
