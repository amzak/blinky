use crate::peripherals::bma423ex::{AxesConfig, Bma423Ex, InterruptIOCtlFlags};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use blinky_shared::error::Error;
use bma423::{Bma423, FeatureInterruptStatus};
use embedded_hal_compat::{Reverse, ReverseCompat};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::i2c::I2cDriver;
use log::{debug, info, trace};
use tokio::time::{sleep, Duration};

pub struct Accelerometer<'a> {
    accel_base: Bma423<Reverse<I2cProxyAsync<I2cDriver<'a>>>>,
    accel_ex: Bma423Ex<I2cProxyAsync<I2cDriver<'a>>>,
    proxy: I2cProxyAsync<I2cDriver<'a>>,
}

pub struct Thermometer<'a> {
    accel_ex: Bma423Ex<I2cProxyAsync<I2cDriver<'a>>>,
}

impl<'a> Accelerometer<'a> {
    pub async fn create(
        proxy: I2cProxyAsync<I2cDriver<'a>>,
        proxy_ex: I2cProxyAsync<I2cDriver<'a>>,
    ) -> Result<Accelerometer<'a>, Error> {
        let thermo_proxy = proxy.clone();
        let mut accel = Bma423::new_with_address(proxy.reverse(), 0x18);
        let mut accel_ex = Bma423Ex::new(proxy_ex);

        let mut delay = Ets;

        let mut counter = 0;

        loop {
            if let Err(err) = accel_ex.reset_and_init(&mut delay) {
                info!("err {}", err.cause());
                if counter > 3 {
                    return Err(Error::from(err.to_string().as_str()));
                }

                sleep(Duration::from_millis(20)).await;
                counter += 1;
                continue;
            } else {
                break;
            }
        }

        let internal_status = accel_ex.read_internal_status().unwrap();
        info!("internal_status = {}", internal_status);

        accel
            .set_accel_config(
                bma423::AccelConfigOdr::Odr100,
                bma423::AccelConfigBandwidth::NormAvg4,
                bma423::AccelConfigPerfMode::CicAvg,
                bma423::AccelRange::Range2g,
            )
            .unwrap();

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

        let accel = Accelerometer {
            accel_base: accel,
            accel_ex,
            proxy: thermo_proxy,
        };

        Ok(accel)
    }

    pub fn read_interrupt_status(&mut self) -> FeatureInterruptStatus {
        self.accel_ex.read_int0_status().unwrap()
    }

    pub fn get_thermometer(&self) -> Thermometer<'a> {
        Thermometer {
            accel_ex: Bma423Ex::new(self.proxy.clone()),
        }
    }
}

impl<'a> Thermometer<'a> {
    pub fn read_temperature(&mut self) -> f32 {
        let t = self.accel_ex.read_temperature().unwrap();

        return t;
    }
}
