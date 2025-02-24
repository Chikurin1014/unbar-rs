use bno055::Bno055;
use core::result::Result;
use embassy_time::{Delay, Duration};
use esp_hal::{
    gpio, i2c,
    ledc::{self, channel::ChannelIFace as _, timer::TimerIFace},
    Async,
};
use log::debug;

pub struct Imu<'a> {
    pub bno055: Bno055<i2c::master::I2c<'a, Async>>,
    reset: gpio::Output<'a>,
}

impl<'a> Imu<'a> {
    pub async fn new(
        i2c: i2c::master::I2c<'a, Async>,
        reset: gpio::Output<'a>,
        delay: &mut Delay,
    ) -> Self {
        debug!("Init IMU");
        let bno055 = Bno055::new(i2c).with_alternative_address();
        let mut imu = Self { bno055, reset };
        imu.hard_reset().await;
        while let Err(e) = imu.init(delay).await {
            debug!("IMU init failed: {:?}", e);
        }
        debug!("IMU init success");
        imu
    }

    async fn init(&mut self, delay: &mut Delay) -> Result<(), bno055::Error<i2c::master::Error>> {
        self.bno055.init(delay)?;
        self.bno055.set_external_crystal(false, delay)?;
        self.bno055
            .set_mode(bno055::BNO055OperationMode::NDOF, delay)?;
        Ok(())
    }

    pub async fn hard_reset(&mut self) {
        self.reset.set_low();
        embassy_time::Timer::after(Duration::from_micros(1)).await;
        self.reset.set_high();
    }
}

pub struct Motor<'a, S: ledc::timer::TimerSpeed> {
    dir1: gpio::Output<'a>,
    dir2: gpio::Output<'a>,
    pwm_ch: ledc::channel::Channel<'a, S>,
}

impl<'a, S: ledc::timer::TimerSpeed> Motor<'a, S> {
    pub fn new(
        dir1: gpio::Output<'a>,
        dir2: gpio::Output<'a>,
        pwm_ch: ledc::channel::Channel<'a, S>,
    ) -> Self {
        Self { dir1, dir2, pwm_ch }
    }

    pub fn attach_timer(
        &mut self,
        timer: &'a impl TimerIFace<S>,
    ) -> Result<(), ledc::channel::Error> {
        self.pwm_ch.configure(ledc::channel::config::Config {
            timer,
            duty_pct: 0,
            pin_config: ledc::channel::config::PinConfig::PushPull,
        })
    }

    pub fn set_speed(&mut self, speed: i16) -> Result<(), ledc::channel::Error> {
        let sgn = speed.signum();
        let abs = speed.abs() as u8;

        match sgn {
            0 => {
                self.dir1.set_high();
                self.dir2.set_high();
            }
            1 => {
                self.dir1.set_high();
                self.dir2.set_low();
            }
            -1 => {
                self.dir1.set_low();
                self.dir2.set_high();
            }
            _ => unreachable!(),
        }

        self.pwm_ch.set_duty(abs)
    }

    pub fn stop(&mut self) -> Result<(), ledc::channel::Error> {
        self.dir1.set_low();
        self.dir2.set_low();
        self.pwm_ch.set_duty(0)
    }

    pub fn is_stop(&self) -> bool {
        self.dir1.is_set_low() && self.dir2.is_set_low()
    }
}
