use blinky_shared::error::Error;
use bma423::{AccelConfigOdr, Bma423, Config, FeatureInterruptStatus, FullPower, InterruptStatus};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::i2c::I2cDriver;
use log::{debug, error, info, warn};
use peripherals::{
    bma423ex::{AxesConfig, Bma423Ex, InterruptIOCtlFlags},
    i2c_proxy_async::I2cProxyAsync,
};
use tokio::time::{sleep, Duration};

pub struct Accelerometer<'a> {
    accel_base: Bma423<I2cProxyAsync<I2cDriver<'a>>, FullPower>,
    pub temperature: i32,
}

pub struct Thermometer<'a> {
    accel_ex: Bma423Ex<I2cProxyAsync<I2cDriver<'a>>>,
}

impl<'a> Accelerometer<'a> {
    pub async fn create(
        proxy: I2cProxyAsync<I2cDriver<'a>>,
        proxy_ex: I2cProxyAsync<I2cDriver<'a>>,
    ) -> Result<Accelerometer<'a>, Error> {
        let mut accel_ex = Bma423Ex::new(proxy_ex);

        let mut delay = Ets;

        let mut counter = 0;

        let mut accel_base_initialized_opt = None;

        loop {
            let mut config = Config::default();

            config.sample_rate = AccelConfigOdr::Odr12p5;

            let accel_base = Bma423::new_with_address(proxy.clone(), config, 0x18);
            let accel_base_initialized_res = accel_base.init(&mut delay);

            match accel_base_initialized_res {
                Ok(initialized) => {
                    accel_base_initialized_opt = Some(initialized);
                    break;
                }
                Err(err) => {
                    let err_message = format!("err {:?}", err);

                    warn!("{}", err_message);

                    if counter > 2 {
                        error!("{}", err_message);
                        return Err(Error::from(err_message.as_str()));
                    }

                    sleep(Duration::from_millis(20)).await;
                    counter += 1;
                }
            }
        }

        let internal_status = accel_ex.read_internal_status().unwrap();
        info!("internal_status = {}", internal_status);

        let axes_config = AxesConfig {
            x_axis: 0,
            x_axis_inv: 0,
            y_axis: 1,
            y_axis_inv: 1,
            z_axis: 2,
            z_axis_inv: 1,
        };

        accel_ex.remap_axes(axes_config).unwrap();
        accel_ex.enable_wrist_tilt().unwrap();

        let int1_cfg = accel_ex
            .configure_int1_io_ctrl(InterruptIOCtlFlags::OutputEn | InterruptIOCtlFlags::Od)
            .unwrap();
        debug!("int1_cfg = {}", int1_cfg);

        accel_ex
            .map_int1_feature_interrupt(
                FeatureInterruptStatus::WristWear, /* | FeatureInterruptStatus::AnyMotion*/
                true,
            )
            .unwrap();

        let feature_config = accel_ex.get_feature_config().unwrap();
        debug!("feature_config = {:02X?}", feature_config);

        let temperature = accel_ex.read_temperature().unwrap();

        let accel = Accelerometer {
            accel_base: accel_base_initialized_opt.unwrap(),
            temperature,
        };

        Ok(accel)
    }

    pub fn read_interrupt_status(&mut self) -> InterruptStatus {
        self.accel_base.read_interrupt_status().unwrap()
    }

    pub fn read_accel(&mut self) -> (f32, f32, f32) {
        self.accel_base.accel_norm_int().unwrap()
    }
    // pub fn get_thermometer(&self) -> Thermometer<'a> {
    //     Thermometer {
    //         accel_ex: Bma423Ex::new(self.proxy.clone()),
    //     }
    // }
}

impl<'a> Thermometer<'a> {
    pub fn read_temperature(&mut self) -> i32 {
        let t = self.accel_ex.read_temperature().unwrap();
        return t;
    }
}
