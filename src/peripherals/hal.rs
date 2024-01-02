use crate::modules::calendar_module::CalendarEvent;
use crate::modules::reference_time::ReferenceData;
use crate::modules::touch_module::TouchPosition;
use crate::peripherals::i2c_management::I2cManagement;
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::touchpad::TouchpadConfig;
use crate::persistence::{PersistenceUnit, PersistenceUnitKind};
use esp_idf_hal::gpio::{Gpio25, Gpio26, IOPin};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::units::FromValueType;

use time::OffsetDateTime;

pub struct HAL<'d> {
    i2c_manager: I2cManagement<'d>,
    pub config: HalConfig,
}

pub struct HalConfig {
    pub backlight: i32,
    pub touch_interrupt_pin: i32,
    pub touch_reset_pin: i32,
}

pub struct PinConfig {
    pub backlight: i32,
}

impl<'d> HAL<'d> {
    fn init_i2c(i2c: I2C0) -> I2cManagement<'d> {
        let scl = unsafe { Gpio25::new() };
        let sda = unsafe { Gpio26::new() };
        let config = I2cConfig::new().baudrate(100.kHz().into());

        I2cManagement::create(i2c, scl.downgrade(), sda.downgrade(), config)
    }

    pub fn new(config: HalConfig, peripherals: I2C0) -> HAL<'d> {
        Self {
            i2c_manager: Self::init_i2c(peripherals),
            config,
        }
    }

    pub fn get_i2c_proxy_async(&self) -> I2cProxyAsync<I2cDriver<'d>> {
        return self.i2c_manager.get_proxy_ref_async();
    }

    pub fn get_touch_config(&self) -> TouchpadConfig {
        TouchpadConfig {
            interrupt_pin: self.config.touch_interrupt_pin,
            reset_pin: self.config.touch_reset_pin,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum WakeupCause {
    Undef,
    All,
    Ext0,
    Ext1,
    Timer,
    Touch,
    Ulp,
}

#[derive(Clone, Debug)]
pub enum Commands {
    RequestReferenceData,
    SyncRtc,
    SyncCalendar,
    GetTimeNow,
    GetReferenceTime,
    SetTime(OffsetDateTime),
    StartDeepSleep,
    PauseRendering,
    ResumeRendering,
    GetTemperature,
    Persist(PersistenceUnit),
    Restore(PersistenceUnitKind),
}

#[derive(Clone, Debug)]
pub enum Events {
    TimeNow(OffsetDateTime),
    Timezone(i32),
    BluetoothConnected,
    ReferenceData(ReferenceData),
    ReferenceTime(OffsetDateTime),
    Wakeup(WakeupCause),
    TouchOrMove,
    TouchPos(TouchPosition),
    IncomingData(Vec<u8>),
    Temperature(f32),
    BatteryLevel(u16),
    Charging(bool),
    InSync(bool),
    ReferenceCalendarEvent(CalendarEvent),
    ReferenceCalendarEventsCount(i32),
    CalendarEvent(CalendarEvent),
    Restored(PersistenceUnit),
    Term,
}
