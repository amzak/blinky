use std::convert::Infallible;

use blinky_shared::{
    display_interface::{ClockDisplayInterface, LayerType, RenderMode},
    fasttrack::FastTrackRtcData,
    modules::renderer::{Renderer, TimeViewModel},
};
use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_hal::{gpio::OutputPin, i2c::I2cDriver, peripheral::Peripheral, spi::SpiAnyPins};
use peripherals::i2c_proxy_async::I2cProxyAsync;
use time::UtcOffset;

use crate::peripherals::{
    display::ClockDisplay, output::PinOutput, pins::mapping::PinsMapping, rtc::Rtc,
    rtc_memory::UTC_OFFSET,
};

pub struct RtcDisplayFastTrack {}

pub struct FastTrackResult<'a, TDisplay>
where
    TDisplay: ClockDisplayInterface<Error = Infallible, ColorModel = Rgb565>,
{
    pub rtc: Rtc<'a>,
    pub display: TDisplay,
    pub rtc_data: FastTrackRtcData,
}

impl RtcDisplayFastTrack {
    fn missing_timezone_info() -> bool {
        unsafe { UTC_OFFSET.is_none() }
    }

    fn get_timezone() -> UtcOffset {
        unsafe { UTC_OFFSET.unwrap() }
    }

    pub fn run_and_decompose<'a, TSpi, TBacklightPin, TSpiDC, TSpiRst, PM>(
        spi: impl Peripheral<P = TSpi> + 'static,
        i2c_proxy: I2cProxyAsync<I2cDriver<'a>>,
        pins_mapping: &mut PM,
    ) -> FastTrackResult<'a, ClockDisplay<'a, TSpiDC, TSpiRst>>
    where
        TSpi: SpiAnyPins,
        TBacklightPin: OutputPin,
        TSpiDC: embedded_hal::digital::OutputPin + Send + 'static,
        TSpiRst: embedded_hal::digital::OutputPin + Send + 'static,
        PM: PinsMapping<TBacklightPin = TBacklightPin, TSpiDC = TSpiDC, TDisplayRst = TSpiRst>,
    {
        let backlight_pin = pins_mapping.get_backlight_pin();
        let backlight_pin_index = backlight_pin.pin();

        let _ = PinOutput::create(backlight_pin_index, true);

        let mut rtc = Rtc::create(i2c_proxy);
        let mut display = ClockDisplay::<'_, TSpiDC, TSpiRst>::create_hal(spi, pins_mapping);

        let alarm_status = rtc.get_alarm_status();

        if Self::missing_timezone_info() {
            return FastTrackResult {
                rtc,
                display,
                rtc_data: FastTrackRtcData {
                    now: None,
                    alarm_status,
                },
            };
        }

        let timezone = Self::get_timezone();

        let now_local = rtc.get_now_utc().assume_offset(timezone);

        let time_view_model = TimeViewModel {
            time: Some(now_local),
        };

        display.render(LayerType::Clock, RenderMode::Ammend, |mut frame| {
            Renderer::<ClockDisplay<'_, TSpiDC, TSpiRst>>::render_datetime(
                &mut frame,
                &time_view_model,
            );

            frame
        });

        display.commit(LayerType::Clock.into());

        return FastTrackResult {
            rtc,
            display,
            rtc_data: FastTrackRtcData {
                now: Some(now_local),
                alarm_status,
            },
        };
    }
}
