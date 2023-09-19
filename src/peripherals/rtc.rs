use esp_idf_hal::i2c::I2cDriver;
use pcf8563::{DateTime, PCF8563};
use embedded_hal_compat::{Reverse, ReverseCompat};
use time::{Date, Month, OffsetDateTime, UtcOffset};

use crate::peripherals::i2c_proxy::I2cProxy;

pub type RtcDevice<'a> = PCF8563<Reverse<I2cProxy<I2cDriver<'a>>>>;

pub struct Rtc<'a> {
    rtc: RtcDevice<'a>
}

impl<'a> Rtc<'a> {
    pub fn create(proxy: I2cProxy<I2cDriver<'a>>) -> Self {
        let mut rtc = PCF8563::new(proxy.reverse());

        Self {
            rtc
        }
    }

    pub fn get_now(&mut self) -> OffsetDateTime {
        let offset = UtcOffset::from_hms(3, 0, 0).unwrap();

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
            .unwrap()
            .assume_offset(offset);

        datetime
    }

    pub fn test(&mut self) {
        self.rtc.get_datetime().unwrap();
        let offset = UtcOffset::from_hms(2, 0, 0).unwrap();

        let datetime_rtc = DateTime {
            year: 23,
            month: 1,
            weekday: 1,
            day: 1,
            hours: 0,
            minutes: 0,
            seconds: 0,
        };

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
        .unwrap()
        .assume_offset(offset);

    }
}