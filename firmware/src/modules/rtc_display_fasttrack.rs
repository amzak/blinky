use blinky_shared::{
    display_interface::{ClockDisplayInterface, LayerType, RenderMode},
    modules::renderer::{Renderer, TimeViewModel},
};
use esp_idf_hal::i2c::I2cDriver;
use time::{OffsetDateTime, UtcOffset};

use crate::peripherals::{
    display::ClockDisplay, hal::HalConfig, i2c_proxy_async::I2cProxyAsync, output::PinOutput,
    rtc::Rtc, rtc_memory::UTC_OFFSET,
};

pub struct RtcDisplayFastTrack {}

pub struct FastTrackResult<'a> {
    pub rtc: Rtc<'a>,
    pub display: ClockDisplay<'a>,
    pub now: Option<OffsetDateTime>,
}

impl RtcDisplayFastTrack {
    fn missing_timezone_info() -> bool {
        unsafe { UTC_OFFSET.is_none() }
    }

    fn get_timezone() -> UtcOffset {
        unsafe { UTC_OFFSET.unwrap() }
    }

    pub fn run_and_decompose<'a>(
        config: HalConfig,
        i2c_proxy: I2cProxyAsync<I2cDriver<'a>>,
    ) -> FastTrackResult<'a> {
        let _ = PinOutput::create(config.backlight, true);

        let mut rtc = Rtc::create(i2c_proxy);
        let mut display = ClockDisplay::create();

        if Self::missing_timezone_info() {
            return FastTrackResult {
                rtc,
                display,
                now: None,
            };
        }

        let timezone = Self::get_timezone();

        let now_local = rtc.get_now_utc().assume_offset(timezone);

        let time_view_model = TimeViewModel {
            time: Some(now_local),
        };

        display.render(LayerType::Clock, RenderMode::Ammend, |mut frame| {
            Renderer::<ClockDisplay>::render_datetime(&mut frame, &time_view_model);

            frame
        });

        display.commit(LayerType::Clock.into());

        return FastTrackResult {
            rtc,
            display,
            now: Some(now_local),
        };
    }
}
