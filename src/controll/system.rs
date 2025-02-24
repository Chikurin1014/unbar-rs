use esp_hal::time::Instant;
use num_traits::real::Real as _;

pub struct System {
    initial_time: Instant,
    time: Instant,
}

impl System {
    pub fn new() -> Self {
        Self {
            initial_time: esp_hal::time::now(),
            time: esp_hal::time::now(),
        }
    }
}

pub trait SystemIFace {
    type Input;
    type Output;

    fn step(&mut self, input: &Self::Input) -> Self::Output;
}

impl SystemIFace for System {
    type Input = ();
    type Output = super::output::MotorSpeed;

    fn step(&mut self, _input: &Self::Input) -> Self::Output {
        let new_time = esp_hal::time::now();
        let output = Self::Output {
            left: (((self.time - self.initial_time).to_millis() as f32 / 1000f32).sin() * 100f32)
                as i16,
            right: (((self.time - self.initial_time).to_millis() as f32 / 1000f32).cos() * 100f32)
                as i16,
        };
        self.time = new_time;

        output
    }
}
