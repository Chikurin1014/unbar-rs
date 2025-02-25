use embassy_time::Delay;
use esp_hal::{
    gpio, i2c,
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
    left_motor_dir1: Option<gpio::Output<'a>>,
    left_motor_dir2: Option<gpio::Output<'a>>,
    left_motor_pwm_ch: Option<ledc::channel::Channel<'a, HighSpeed>>,
    right_motor_dir1: Option<gpio::Output<'a>>,
    right_motor_dir2: Option<gpio::Output<'a>>,
    right_motor_pwm_ch: Option<ledc::channel::Channel<'a, HighSpeed>>,
    i2c: Option<i2c::master::I2c<'a, Async>>,
    imu_reset: Option<gpio::Output<'a>>,
    delay: Option<Delay>,
}

#[derive(Debug)]
pub enum HardwareBuildError {
    MissingLeftMotorDir1,
    MissingLeftMotorDir2,
    MissingLeftMotorPwmCh,
    MissingRightMotorDir1,
    MissingRightMotorDir2,
    MissingRightMotorPwmCh,
    MissingI2c,
    MissingImuReset,
    MissingDelay,
}

impl<'a> HardwareBuilder<'a> {
    pub async fn build(self) -> Result<Hardware<'a>, HardwareBuildError> {
        Ok(Hardware {
            left_motor: Motor::new(
                self.left_motor_dir1
                    .ok_or(HardwareBuildError::MissingLeftMotorDir1)?,
                self.left_motor_dir2
                    .ok_or(HardwareBuildError::MissingLeftMotorDir2)?,
                self.left_motor_pwm_ch
                    .ok_or(HardwareBuildError::MissingLeftMotorPwmCh)?,
            ),
            right_motor: Motor::new(
                self.right_motor_dir1
                    .ok_or(HardwareBuildError::MissingRightMotorDir1)?,
                self.right_motor_dir2
                    .ok_or(HardwareBuildError::MissingRightMotorDir2)?,
                self.right_motor_pwm_ch
                    .ok_or(HardwareBuildError::MissingRightMotorPwmCh)?,
            ),
            imu: Imu::new(
                self.i2c.ok_or(HardwareBuildError::MissingI2c)?,
                self.imu_reset.ok_or(HardwareBuildError::MissingImuReset)?,
                &mut self.delay.ok_or(HardwareBuildError::MissingDelay)?.clone(),
            )
            .await,
        })
    }
    pub fn left_motor(
        mut self,
        dir1: gpio::Output<'a>,
        dir2: gpio::Output<'a>,
        pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
    ) -> Self {
        self.left_motor_dir1 = Some(dir1);
        self.left_motor_dir2 = Some(dir2);
        self.left_motor_pwm_ch = Some(pwm_ch);
        self
    }
    pub fn right_motor(
        mut self,
        dir1: gpio::Output<'a>,
        dir2: gpio::Output<'a>,
        pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
    ) -> Self {
        self.right_motor_dir1 = Some(dir1);
        self.right_motor_dir2 = Some(dir2);
        self.right_motor_pwm_ch = Some(pwm_ch);
        self
    }
    pub fn imu(mut self, i2c: i2c::master::I2c<'a, Async>, reset: gpio::Output<'a>) -> Self {
        self.i2c = Some(i2c);
        self.imu_reset = Some(reset);
        self
    }
    pub fn delay(mut self, delay: Delay) -> Self {
        self.delay = Some(delay);
        self
    }
}

impl<'a> Hardware<'a> {
    // pub fn builder(
    //     left_motor_dir1: gpio::Output<'a>,
    //     left_motor_dir2: gpio::Output<'a>,
    //     left_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
    //     right_motor_dir1: gpio::Output<'a>,
    //     right_motor_dir2: gpio::Output<'a>,
    //     right_motor_pwm_ch: ledc::channel::Channel<'a, HighSpeed>,
    //     i2c: i2c::master::I2c<'a, Async>,
    //     imu_reset: gpio::Output<'a>,
    //     delay: Delay,
    // ) -> HardwareBuilder<'a> {
    //     HardwareBuilder {
    //         left_motor_dir1,
    //         left_motor_dir2,
    //         left_motor_pwm_ch,
    //         right_motor_dir1,
    //         right_motor_dir2,
    //         right_motor_pwm_ch,
    //         i2c,
    //         imu_reset,
    //         delay: delay,
    //     }
    pub fn builder() -> HardwareBuilder<'a> {
        HardwareBuilder {
            left_motor_dir1: None,
            left_motor_dir2: None,
            left_motor_pwm_ch: None,
            right_motor_dir1: None,
            right_motor_dir2: None,
            right_motor_pwm_ch: None,
            i2c: None,
            imu_reset: None,
            delay: None,
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
