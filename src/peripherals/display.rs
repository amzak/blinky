use std::{ error::Error };
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::{gpio, spi};
use esp_idf_hal::gpio::{AnyIOPin, Gpio13, Gpio14, Gpio15, Gpio19, Gpio27, InputOutput, Output, OutputPin, PinDriver};
use esp_idf_hal::spi::config::DriverConfig;
use esp_idf_hal::spi::{Dma, SPI2, SpiDeviceDriver, SpiDriver, SpiSingleDeviceDriver};
use esp_idf_hal::units::FromValueType;
use mipidsi::{Builder, Display};
use time::macros::{datetime, format_description, offset};

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13},
        iso_8859_16::FONT_10X20,
        MonoTextStyle,
    },
    prelude::{*, DrawTarget},
    text::Text,
};
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use embedded_graphics::text::Alignment;

use mipidsi::models::GC9A01;
use time::OffsetDateTime;

pub type EspSpi1InterfaceNoCS<'d> = SPIInterfaceNoCS<SpiSingleDeviceDriver<'d>, PinDriver<'d, Gpio19, InputOutput>>;
pub type DisplaySPI2<'d> = Display<EspSpi1InterfaceNoCS<'d>, GC9A01, PinDriver<'d, Gpio27, InputOutput>>;

pub struct ClockDisplay<'d> {
    display: DisplaySPI2<'d>
}

impl<'d> ClockDisplay<'d> {
    pub fn create() -> Self {
        let mut delay = Ets;
        let spi = unsafe { SPI2::new() };
        let cs = unsafe { Gpio15::new() };
        let sclk = unsafe { Gpio14::new() };
        let sdo = unsafe { Gpio13::new() };
        let rst = PinDriver::input_output_od(unsafe { Gpio27::new() }).unwrap();
        let dc = PinDriver::input_output_od(unsafe { Gpio19::new() }).unwrap();

        let config = DriverConfig {
            dma: Dma::Disabled,
            intr_flags: Default::default(),
        };

        let driver = SpiDriver::new(spi, sclk, sdo, None::<AnyIOPin>, &config).unwrap();

        let spi_config = esp_idf_hal::spi::config::Config::default()
            .baudrate(20_000_000.Hz())
            .write_only(true);

        let spi = SpiDeviceDriver::new(driver, Some(cs), &spi_config).unwrap();

        let di = SPIInterfaceNoCS::new(spi, dc);

        let mut display = Builder::gc9a01(di)
            .init(&mut delay, Some(rst))
            .map_err(|_| Box::<dyn Error>::from("display init"))
            .unwrap();

        Self { display }
    }

    pub fn clear(&mut self) {
        self.display.clear(Rgb565::WHITE)
            .map_err(|_| Box::<dyn Error>::from("clear display"))
            .unwrap();
    }

    pub fn text_aligned(&mut self, text: &str, coord: Point, style: MonoTextStyle<Rgb565>, alignment: Alignment) {
        Text::with_alignment(text, coord, style, alignment)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn text(&mut self, text: &str, coord: Point) {
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new(text, coord, style)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn circle(&mut self, coord: Point, diameter: u32, style: PrimitiveStyle<Rgb565>) {
        Circle::new(coord, 5)
            .into_styled(style)
            .draw(&mut self.display).unwrap();
    }

    pub fn test_show_time(&mut self, datetime: OffsetDateTime) {
        let template = format_description!(
            version = 2,
            "[hour repr:24]:[minute]:[second]"
        );

        let text = datetime.format(&template).unwrap();
        let style_time = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);

        self.text_aligned(&text, Point::new(120, 120), style_time, embedded_graphics::text::Alignment::Center);

    }
}