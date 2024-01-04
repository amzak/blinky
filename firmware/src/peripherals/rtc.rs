use embedded_hal_compat::{Reverse, ReverseCompat};
use esp_idf_hal::i2c::{I2cDriver, I2cError};
use pcf8563::Error::I2C;
use pcf8563::{DateTime, PCF8563};
use time::{Date, Month, PrimitiveDateTime};

use crate::error::Error;
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;

pub type RtcDevice<'a> = PCF8563<Reverse<I2cProxyAsync<I2cDriver<'a>>>>;

pub struct Rtc<'a> {
    rtc: RtcDevice<'a>,
}

impl<'a> Rtc<'a> {
    pub fn create(proxy: I2cProxyAsync<I2cDriver<'a>>) -> Self {
        let mut rtc = PCF8563::new(proxy.reverse());
        rtc.rtc_init().unwrap();

        Self { rtc }
    }

    pub fn get_now_utc(&mut self) -> PrimitiveDateTime {
        let datetime_rtc = self.rtc.get_datetime().unwrap();

        let datetime = Date::from_calendar_date(
            datetime_rtc.year as i32 + 2000,
            Month::try_from(datetime_rtc.month).unwrap(),
            datetime_rtc.day,
        )
        .unwrap()
        .with_hms(
            datetime_rtc.hours,
            datetime_rtc.minutes,
            datetime_rtc.seconds,
        )
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
            I2C(i2c_err) => Error::from(i2c_err),
            pcf8563::Error::InvalidInputData => Error::from("invalid input data"),
        });
    }
}
