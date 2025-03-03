use std::{
    cell::RefCell,
    convert::Infallible,
    rc::Rc,
    sync::{Arc, Mutex},
};

use blinky_shared::{
    display_interface::{ClockDisplayInterface, LayerType, RenderMode},
    fasttrack::FastTrackRtcData,
    modules::{
        fonts_set::FontSet466,
        icon_set::IconSet,
        icon_set_466::IconsSet466,
        renderer::{Renderer, TimeViewModel},
    },
};
use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_hal::{
    gpio::{Output, OutputPin, PinDriver},
    i2c::I2cDriver,
    peripheral::Peripheral,
    spi::SpiAnyPins,
};
use peripherals::i2c_proxy_async::I2cProxyAsync;
use time::UtcOffset;

use crate::peripherals::{display::ClockDisplay, rtc::Rtc, rtc_memory::UTC_OFFSET};
use peripherals::pins::mapping::PinsMapping;

pub struct RtcDisplayFastTrack {}

pub struct FastTrackResult<'a, TDisplay, TBacklightPin>
where
    TDisplay: ClockDisplayInterface<Error = Infallible, ColorModel = Rgb565>,
    TBacklightPin: OutputPin,
{
    pub rtc: Rtc<'a>,
    pub display: TDisplay,
    pub backlight: Option<PinDriver<'a, TBacklightPin, Output>>,
    pub rtc_data: FastTrackRtcData,
}

impl RtcDisplayFastTrack {
    fn missing_timezone_info() -> bool {
        unsafe { UTC_OFFSET.is_none() }
    }

    fn get_timezone() -> UtcOffset {
        unsafe { UTC_OFFSET.unwrap() }
    }

    pub fn run_and_decompose<'a, TSpi, TBacklightPin, TSpiDC, TSpiRst, TEN, PM>(
        spi: impl Peripheral<P = TSpi> + 'static,
        i2c_proxy: I2cProxyAsync<I2cDriver<'a>>,
        pins_mapping: Arc<Mutex<PM>>,
    ) -> FastTrackResult<'a, ClockDisplay<'a, TSpiDC, TSpiRst, TEN>, TBacklightPin>
    where
        TSpi: SpiAnyPins,
        TBacklightPin: OutputPin,
        TSpiDC: embedded_hal::digital::OutputPin + Send + 'static,
        TSpiRst: embedded_hal::digital::OutputPin + Send + 'static,
        TEN: OutputPin,
        PM: PinsMapping<
            TBacklightPin = TBacklightPin,
            TSpiDC = TSpiDC,
            TDisplayRst = TSpiRst,
            TDisplayEn = TEN,
        >,
    {
        let backlight_pin = pins_mapping.lock().unwrap().get_backlight_pin();

        let mut backlight_pin_driver = None;

        if backlight_pin.is_some() {
            let mut backlight_pin = PinDriver::output(backlight_pin.unwrap()).unwrap();
            backlight_pin.set_high().unwrap();
            backlight_pin_driver = Some(backlight_pin);
        }

        let mut rtc = Rtc::create(i2c_proxy);
        let mut display =
            ClockDisplay::<'_, TSpiDC, TSpiRst, TEN>::create_hal(spi, pins_mapping.clone());

        let alarm_status = rtc.get_alarm_status();

        if Self::missing_timezone_info() {
            return FastTrackResult {
                rtc,
                display,
                backlight: backlight_pin_driver,
                rtc_data: FastTrackRtcData {
                    now: None,
                    alarm_status,
                },
            };
        }

        let timezone = Self::get_timezone();

        let now_local = rtc.get_now_utc().unwrap().assume_offset(timezone);

        let time_view_model = TimeViewModel {
            time: Some(now_local),
        };

        display.render(LayerType::Clock, RenderMode::Ammend, |mut frame| {
            Renderer::<ClockDisplay<'_, TSpiDC, TSpiRst, TEN>, FontSet466, IconsSet466>::render_datetime(
                &mut frame,
                &time_view_model,
            );

            frame
        });

        display.commit(LayerType::Clock.into());

        return FastTrackResult {
            rtc,
            display,
            backlight: backlight_pin_driver,
            rtc_data: FastTrackRtcData {
                now: Some(now_local),
                alarm_status,
            },
        };
    }
}
