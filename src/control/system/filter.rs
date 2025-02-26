use num_traits::real::Real;

pub trait LowPassFilterIFace {
    type Value: Real;
    type Time: Real;
    fn filter(
        &mut self,
        input: Self::Value,
        time: Self::Time,
        time_const: Self::Time,
    ) -> Self::Value;
    fn get_current(&self) -> Self::Value;
}

#[derive(Debug, Clone)]
pub struct LowPassFilter<V: Real> {
    value: V,
    time: V,
}

impl<V: Real> LowPassFilter<V> {
    pub fn new(initial_value: V) -> Self {
        Self {
            value: initial_value,
            time: V::zero(),
        }
    }
}

impl<V: Real> Default for LowPassFilter<V> {
    fn default() -> Self {
        Self::new(V::zero())
    }
}

impl<V: Real> LowPassFilterIFace for LowPassFilter<V> {
    type Value = V;
    type Time = V;

    fn filter(
        &mut self,
        input: Self::Value,
        time: Self::Time,
        time_const: Self::Time,
    ) -> Self::Value {
        let dt = time - self.time;
        let t = time_const.max(dt);

        self.value = (V::one() - dt / t) * self.value + dt / t * input;
        self.time = time;

        self.value
    }

    fn get_current(&self) -> Self::Value {
        self.value
    }
}
