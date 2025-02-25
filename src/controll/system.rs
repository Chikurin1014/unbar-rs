use core::f32::consts::PI;

use esp_hal::time::Instant;
use num_traits::real::Real as _;

#[derive(Debug, Clone, Copy)]
pub struct System {
    /// 制御時刻
    time: Instant,
    /// 目標姿勢角と実際の姿勢角との差分
    error: f32,
}

impl System {
    pub fn new() -> Self {
        Self {
            time: esp_hal::time::now(),
            error: 0.0,
        }
    }
}

pub trait SystemIFace {
    type Input;
    type Output;

    fn step(&mut self, input: &Self::Input) -> Self::Output;
}

impl SystemIFace for System {
    type Input = bno055::mint::Vector3<f32>;
    type Output = super::output::MotorSpeed;

    fn step(&mut self, input: &Self::Input) -> Self::Output {
        let time = esp_hal::time::now();

        // t [s]
        let _t = time.duration_since_epoch().to_micros() as f32 / 1e6;
        let dt = (time - self.time).to_micros() as f32 / 1e6;

        // target [m/s^2]
        let target = bno055::mint::Vector3::from_slice(&[0.0, 0.0, 1.0]);

        // e [rad]
        let e = target.y.atan2(target.z) - input.y.atan2(input.z);
        let de = e - self.error;

        const KP: f32 = 6.0 / PI;
        const TD: f32 = 0.0;
        const KD: f32 = TD * KP;

        let left = -((KP * e + KD * de / dt) * 100.0) as i8;
        let right = ((KP * e + KD * de / dt) * 100.0) as i8;

        let output = Self::Output { left, right };

        self.error = e;
        self.time = time;

        output
    }
}
