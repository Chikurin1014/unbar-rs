#![no_std]

use esp_hal::{gpio::GpioPin, i2c::master::I2c, peripherals::Peripherals};

pub struct Unbar<'a> {
    pub left_motor: (GpioPin<1>, GpioPin<2>, GpioPin<3>),
    pub right_motor: (GpioPin<4>, GpioPin<5>, GpioPin<6>),
    pub i2c: I2c<'a, esp_hal::Blocking>,
    pub wifi_controller: esp_wifi::EspWifiController<'a>,
}

impl<'a> Unbar<'a> {
    pub fn new(peripherals: Peripherals) -> Self {
        let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
        esp_hal_embassy::init(timg1.timer0);

        let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
        let wifi_controller = esp_wifi::init(
            timg0.timer0,
            esp_hal::rng::Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap();

        let left_motor = (peripherals.GPIO1, peripherals.GPIO2, peripherals.GPIO3);
        let right_motor = (peripherals.GPIO4, peripherals.GPIO5, peripherals.GPIO6);
        let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
            .unwrap()
            .with_scl(peripherals.GPIO22)
            .with_sda(peripherals.GPIO21);

        Self {
            left_motor,
            right_motor,
            i2c,
            wifi_controller,
        }
    }
}
