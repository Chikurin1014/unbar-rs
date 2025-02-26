use core::{f32::consts::PI, fmt::Display};

use esp_hal::time::Instant;
use num_traits::real::Real as _;

mod filter;

use filter::{LowPassFilter, LowPassFilterIFace as _};

#[derive(Debug, Clone)]
pub struct System {
    /// 制御時刻
    time: Instant,
    /// 目標姿勢角と実際の姿勢角との差分
    error: LowPassFilter<f32>,
    error_diff: LowPassFilter<f32>,
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub time: Instant,
    pub error: f32,
    pub error_diff: f32,
}

impl Display for SystemState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&System> for SystemState {
    fn from(system: &System) -> Self {
        Self {
            time: system.time,
            error: system.error.get_current(),
            error_diff: system.error_diff.get_current(),
        }
    }
}

pub trait SystemIFace<'a> {
    type Input;
    type Output;
    type State: Display + From<&'a System>;

    fn step(&mut self, input: &Self::Input) -> Self::Output;
    fn get_state(&self) -> Self::State;
}

impl System {
    pub fn new() -> Self {
        Self {
            time: esp_hal::time::now(),
            error: LowPassFilter::default(),
            error_diff: LowPassFilter::default(),
        }
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemIFace<'_> for System {
    type Input = bno055::mint::Vector3<f32>;
    type Output = super::output::MotorSpeed;
    type State = SystemState;

    fn step(&mut self, input: &Self::Input) -> Self::Output {
        let time = esp_hal::time::now();

        // t [s]
        let t = time.duration_since_epoch().to_micros() as f32 / 1e6;
        let dt = (time - self.time).to_micros() as f32 / 1e6;

        // target [m/s^2]
        let target = bno055::mint::Vector3::from_slice(&[0.0, -0.1, 1.0]);

        // e [rad]
        const T: f32 = 0.06;
        const T_DIFF: f32 = 0.12;

        let raw_e = target.y.atan2(target.z) - input.y.atan2(input.z);
        let e_prev = self.error.get_current();
        let e = self.error.filter(raw_e, t, T);
        let de = self.error_diff.filter(e - e_prev, t, T_DIFF);

        // PD-control
        const KP: f32 = 6.0 / PI;
        const TD: f32 = 0.03;
        const KD: f32 = TD * KP;

        let raw_left = -((KP * e + KD * de / dt) * 100.0) as i8;
        let raw_right = ((KP * e + KD * de / dt) * 100.0) as i8;

        let left = if raw_left.abs() < 5 { 0 } else { raw_left };
        let right = if raw_right.abs() < 5 { 0 } else { raw_right };

        let output = Self::Output { left, right };

        self.time = time;

        output
    }

    fn get_state(&self) -> SystemState {
        self.into()
    }
}
