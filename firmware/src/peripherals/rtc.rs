use blinky_shared::error::Error;
use embedded_hal_compat::{Reverse, ReverseCompat};
use esp_idf_hal::i2c::{I2cDriver, I2cError};
use log::info;
use pcf8563::Error::I2C;
use pcf8563::{DateTime, PCF8563};
use peripherals::i2c_proxy_async::I2cProxyAsync;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

use super::rtc_memory;

pub type RtcDevice<'a> = PCF8563<Reverse<I2cProxyAsync<I2cDriver<'a>>>>;

pub struct Rtc<'a> {
    rtc: RtcDevice<'a>,
}

impl<'a> Rtc<'a> {
    pub fn create(proxy: I2cProxyAsync<I2cDriver<'a>>) -> Self {
        let mut rtc = PCF8563::new(proxy.reverse());

        if !Self::is_rtc_initialized() {
            info!("rtc init");
            rtc.rtc_init().unwrap();
            Self::set_rtc_initialized();
        }

        Self { rtc }
    }

    fn is_rtc_initialized() -> bool {
        unsafe {
            return rtc_memory::RTC_INITIALIZED;
        }
    }

    fn set_rtc_initialized() {
        unsafe {
            rtc_memory::RTC_INITIALIZED = true;
        }
    }

    pub fn get_now_utc(&mut self) -> PrimitiveDateTime {
        let datetime_rtc = self.rtc.get_datetime().unwrap();

        let seconds = if datetime_rtc.seconds > 59 {
            59 // workaround for 60 seconds case
        } else {
            datetime_rtc.seconds
        };

        let datetime = Date::from_calendar_date(
            datetime_rtc.year as i32 + 2000,
            Month::try_from(datetime_rtc.month).unwrap(),
            datetime_rtc.day,
        )
        .unwrap()
        .with_hms(datetime_rtc.hours, datetime_rtc.minutes, seconds)
        .unwrap();

        datetime
    }

    pub fn set_now_utc(&mut self, now: PrimitiveDateTime) -> Result<(), Error> {
        let year = now.year();
        let rtc_year = if year >= 2000 { year - 2000 } else { 0 };

        let dt = DateTime {
            year: rtc_year as u8,
            month: now.month().into(),
            weekday: now.weekday().number_days_from_monday(),
            day: now.day(),
            hours: now.hour(),
            minutes: now.minute(),
            seconds: now.second(),
        };

        let result = self.rtc.set_datetime(&dt);

        return result.map_err(|err: pcf8563::Error<I2cError>| match err {
            I2C(i2c_err) => Error::from(i2c_err.to_string().as_str()),
            pcf8563::Error::InvalidInputData => Error::from("invalid input data"),
        });
    }

    pub fn set_alarm(&mut self, alarm_at: OffsetDateTime) {
        self.rtc.disable_all_alarms().unwrap();
        self.rtc
            .control_alarm_interrupt(pcf8563::Control::Off)
            .unwrap();
        self.rtc.clear_alarm_flag().unwrap();

        self.rtc
            .timer_interrupt_output(pcf8563::InterruptOutput::Continuous)
            .unwrap();

        self.rtc.set_alarm_day(alarm_at.day()).unwrap();
        self.rtc.set_alarm_hours(alarm_at.hour()).unwrap();
        self.rtc.set_alarm_minutes(alarm_at.minute()).unwrap();
        self.rtc.control_alarm_day(pcf8563::Control::On).unwrap();
        self.rtc.control_alarm_hours(pcf8563::Control::On).unwrap();
        self.rtc
            .control_alarm_minutes(pcf8563::Control::On)
            .unwrap();
        self.rtc
            .control_alarm_interrupt(pcf8563::Control::On)
            .unwrap();
    }

    pub fn get_alarm(&mut self) -> Result<PrimitiveDateTime, Error> {
        let day = self.rtc.get_alarm_day().unwrap();
        let hours = self.rtc.get_alarm_hours().unwrap();
        let minutes = self.rtc.get_alarm_minutes().unwrap();

        Ok(PrimitiveDateTime::new(
            Date::MIN,
            Time::from_hms(hours, minutes, 0).unwrap(),
        ))
    }

    pub fn get_alarm_status(&mut self) -> bool {
        let alarm_flag = self.rtc.get_alarm_flag().unwrap();
        return alarm_flag;
    }

    pub fn reset_alarm(&mut self) {
        self.rtc
            .control_alarm_interrupt(pcf8563::Control::Off)
            .unwrap();

        self.rtc.disable_all_alarms().unwrap();
        self.rtc.clear_alarm_flag().unwrap();
    }
}
