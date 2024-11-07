use std::{thread, time::Duration};

use bma423::{AccelConfigOdr, AccelRange, Bma423, Config, FullPower};
use esp_idf_hal::{
    delay::Ets,
    gpio::{Gpio25, Gpio26, IOPin},
    i2c::{I2c, I2cConfig, I2cDriver, I2cError},
    peripheral::Peripheral,
    units::FromValueType,
};
use log::info;
use peripherals::{
    bma423ex::{AxesConfig, Bma423Ex},
    i2c_management::I2cManagement,
    i2c_proxy_async::I2cProxyAsync,
};

#[derive(Clone)]
struct Bma423Tests<'a> {
    pub proxy: I2cProxyAsync<I2cDriver<'a>>,
    pub proxy_ex: I2cProxyAsync<I2cDriver<'a>>,
}

pub fn run<I2C: I2c>(i2c: impl Peripheral<P = I2C> + 'static) {
    let tests = setup_once(i2c);

    should_init_succefully(tests.clone());

    should_read_acceleration_succefully(tests.clone());

    should_init_bma423_ex(tests.clone());

    should_read_internal_status(tests.clone());

    should_use_bma423_ex_with_library_bma423(tests.clone());

    should_remap_axes(tests.clone());

    should_read_fifo(tests.clone());

    should_get_temperature(tests);
}

fn setup_once<I2C: I2c>(i2c: impl Peripheral<P = I2C> + 'static) -> Bma423Tests<'static> {
    let scl = unsafe { Gpio25::new() };
    let sda = unsafe { Gpio26::new() };
    let config = I2cConfig::new().baudrate(100.kHz().into());

    let i2c_man = I2cManagement::create(i2c, scl.downgrade(), sda.downgrade(), config);

    let proxy = i2c_man.get_proxy_ref_async();
    let proxy_ex = i2c_man.get_proxy_ref_async();

    Bma423Tests { proxy, proxy_ex }
}

fn should_init_succefully(context: Bma423Tests) {
    let accel_base_initialized_res = create_base_bma423(context);

    assert!(accel_base_initialized_res.is_ok());
}

fn should_read_acceleration_succefully(context: Bma423Tests<'_>) {
    let accel_base_initialized_res = create_base_bma423(context);
    let mut accel_base = accel_base_initialized_res.unwrap();

    let mut accel_abs: (f32, f32, f32) = (0f32, 0f32, 0f32);

    for _ in 0..10 {
        accel_abs = accel_base.accel_abs().unwrap();
        info!("accel_abs: {:?}", accel_abs);
        thread::sleep(Duration::from_millis(500));
    }

    assert!((accel_abs.0 + accel_abs.1 + accel_abs.2).abs() > 0f32);
}

fn should_init_bma423_ex(context: Bma423Tests) {
    let mut accel_ex = Bma423Ex::new(context.proxy_ex.clone());

    let mut delay = Ets;
    let init_result = accel_ex.init(&mut delay);

    assert!(init_result.is_ok());
}

fn should_read_internal_status(context: Bma423Tests) {
    let mut accel_ex = create_base_bma423_ex(context);

    let internal_status = accel_ex.read_internal_status().unwrap();
    info!("internal_status = {}", internal_status);

    assert!(internal_status & 1 > 0);
}

fn should_get_temperature(context: Bma423Tests) {
    let mut accel_ex = create_base_bma423_ex(context);

    let temperature = accel_ex.read_temperature();

    assert!(temperature.is_ok());
}

fn create_base_bma423(
    context: Bma423Tests,
) -> Result<bma423::Bma423<I2cProxyAsync<I2cDriver<'_>>, FullPower>, bma423::Error<I2cError>> {
    let mut config = Config::default();

    config.sample_rate = AccelConfigOdr::Odr12p5;

    let accel_base = Bma423::new_with_address(context.proxy, config, 0x18);

    let mut delay = Ets;
    let accel_base_initialized_res = accel_base.init(&mut delay);

    accel_base_initialized_res
}

fn create_base_bma423_ex(context: Bma423Tests) -> Bma423Ex<I2cProxyAsync<I2cDriver<'_>>> {
    let mut accel_ex = Bma423Ex::new(context.proxy_ex.clone());

    let mut delay = Ets;
    let _ = accel_ex.init(&mut delay);

    accel_ex
}

fn should_use_bma423_ex_with_library_bma423(context: Bma423Tests) {
    let proxy_ex = context.proxy_ex.clone();

    let accel_base_initialized_res = create_base_bma423(context);
    let mut accel_base = accel_base_initialized_res.unwrap();
    let mut accel_ex = Bma423Ex::new(proxy_ex);

    let _ = accel_base.accel_abs().unwrap();
    let feature_config_result = accel_ex.get_feature_config();

    assert!(feature_config_result.is_ok());
}

fn should_remap_axes(context: Bma423Tests) {
    let proxy_ex = context.proxy_ex.clone();

    let accel_base_initialized_res = create_base_bma423(context);
    let mut accel_base = accel_base_initialized_res.unwrap();
    let mut accel_ex = Bma423Ex::new(proxy_ex);

    accel_ex.enable_wrist_tilt().unwrap();

    let axes_config = AxesConfig {
        x_axis: 0,
        x_axis_inv: 0,
        y_axis: 1,
        y_axis_inv: 0,
        z_axis: 2,
        z_axis_inv: 0,
    };

    let remap_result = accel_ex.remap_axes(axes_config);

    assert!(remap_result.is_ok());

    let mut accel_abs: (f32, f32, f32) = (0f32, 0f32, 0f32);

    for _ in 0..10 {
        accel_abs = accel_base.accel_abs().unwrap();
        info!("remapped accel_abs: {:?}", accel_abs);
        thread::sleep(Duration::from_millis(500));
    }

    let axes_config = AxesConfig {
        x_axis: 0,
        x_axis_inv: 0,
        y_axis: 1,
        y_axis_inv: 1,
        z_axis: 2,
        z_axis_inv: 0,
    };

    let remap_result = accel_ex.remap_axes(axes_config);

    for _ in 0..10 {
        accel_abs = accel_base.accel_abs().unwrap();
        info!("remapped accel_abs: {:?}", accel_abs);
        thread::sleep(Duration::from_millis(500));
    }

    assert!(remap_result.is_ok());
}

fn should_read_fifo(context: Bma423Tests) {
    let proxy_ex = context.proxy_ex.clone();

    let accel_base_initialized_res = create_base_bma423(context);
    let mut accel_base = accel_base_initialized_res.unwrap();
    let mut accel_ex = Bma423Ex::new(proxy_ex);

    accel_ex.enable_fifo().unwrap();

    info!("fifo enabled");

    let fifo_config = accel_ex.get_fifo_config().unwrap();

    info!("fifo_config = {:02X?}", fifo_config);

    for _i in 0..20 {
        thread::sleep(Duration::from_millis(1000));

        let fifo_length = accel_ex.get_fifo_length().unwrap();

        info!("fifo_length = {}", fifo_length);
    }

    let mut buff: [u8; 100] = [0; 100];

    accel_ex.read_fifo_raw(&mut buff).unwrap();

    let fifo_length = accel_ex.get_fifo_length().unwrap();

    info!("fifo_length = {}", fifo_length);

    let decoded_fifo = decode_fifo(buff.into_iter(), AccelRange::Range4g);

    info!("fifo_data = {:?}", decoded_fifo);
}

fn decode_fifo(buff: impl Iterator<Item = u8>, range: AccelRange) -> Vec<(f32, f32, f32)> {
    let mut result = Vec::new();

    for [x_lsb, x_msb, y_lsb, y_msb, z_lsb, z_msb] in buff.array_chunks() {
        let x: i16 = ((((x_msb as i16) << 8) as i16) | (x_lsb as i16)) / 0x10;
        let y: i16 = ((((y_msb as i16) << 8) as i16) | (y_lsb as i16)) / 0x10;
        let z: i16 = ((((z_msb as i16) << 8) as i16) | (z_lsb as i16)) / 0x10;

        let range = range.as_float();

        let next = (
            lsb_to_ms2(x, range, 12),
            lsb_to_ms2(y, range, 12),
            lsb_to_ms2(z, range, 12),
        );

        result.push(next);
    }

    return result;
}

#[inline(always)]
fn lsb_to_ms2(val: i16, g_range: f32, bit_width: u8) -> f32 {
    let half_scale: f32 = (1 << bit_width) as f32 / 2.0;

    val as f32 * g_range / half_scale
}
