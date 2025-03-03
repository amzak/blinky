use std::{thread::sleep, time::Duration};

use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
use esp_idf_hal::{
    gpio::IOPin,
    i2c::{I2c, I2cConfig, I2cDriver},
    peripheral::Peripheral,
};

use esp_idf_hal::units::FromValueType;

use log::info;
use peripherals::{
    i2c_management::I2cManagement, i2c_proxy_async::I2cProxyAsync, pins::mapping::PinsMapping,
};

#[derive(Clone)]
struct Bmi160Tests<'a> {
    pub proxy: I2cProxyAsync<I2cDriver<'a>>,
}

pub fn run<I2C: I2c>(i2c: impl Peripheral<P = I2C> + 'static, pins_mapping: &mut impl PinsMapping) {
    let tests = setup_once(i2c, pins_mapping);

    should_init_succefully(tests.clone());
}

fn should_init_succefully(context: Bmi160Tests<'_>) {
    let address = SlaveAddr::Alternative(true);
    let mut imu = Bmi160::new_with_i2c(context.proxy, address);

    let id = imu.chip_id().unwrap();
    info!("Chip ID: {}", id);

    imu.set_accel_power_mode(AccelerometerPowerMode::Normal)
        .unwrap();
    imu.set_gyro_power_mode(GyroscopePowerMode::Normal).unwrap();
    loop {
        let data = imu.data(SensorSelector::new().accel().gyro()).unwrap();
        let accel = data.accel.unwrap();
        let gyro = data.gyro.unwrap();
        info!(
            "Accelerometer: x {:5} y {:5} z {:5}, \
         Gyroscope: x {:5} y {:5} z {:5}",
            accel.x, accel.y, accel.z, gyro.x, gyro.y, gyro.z
        );

        sleep(Duration::from_millis(500));
    }
}

fn setup_once<I2C: I2c>(
    i2c: impl Peripheral<P = I2C> + 'static,
    pins_mapping: &mut impl PinsMapping,
) -> Bmi160Tests<'static> {
    let sda_pin = pins_mapping.get_i2c_sda_pin();
    let scl_pin = pins_mapping.get_i2c_scl_pin();

    let config = I2cConfig::new().baudrate(100.kHz().into());

    let i2c_man = I2cManagement::create(i2c, scl_pin.downgrade(), sda_pin.downgrade(), config);

    let proxy = i2c_man.get_proxy_ref_async();

    Bmi160Tests { proxy }
}
