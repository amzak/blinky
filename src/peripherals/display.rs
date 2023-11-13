use std::{ error::Error };
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::spi;
use esp_idf_hal::gpio::{AnyIOPin, Gpio13, Gpio14, Gpio15, Gpio19, Gpio27, InputOutput, PinDriver};
use esp_idf_hal::spi::config::DriverConfig;
use esp_idf_hal::spi::{Dma, SPI2, SpiDeviceDriver, SpiDriver, SpiSingleDeviceDriver};
use esp_idf_hal::units::FromValueType;
use mipidsi::{Builder, Display};

use embedded_graphics::{
    mono_font::{
        ascii::FONT_6X10,
        MonoTextStyle,
    },
    prelude::{*, DrawTarget},
    text::Text,
};
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Alignment;
use embedded_graphics_framebuf::FrameBuf;
use mipidsi::models::GC9A01;

pub type EspSpi1InterfaceNoCS<'d> = SPIInterfaceNoCS<SpiSingleDeviceDriver<'d>, PinDriver<'d, Gpio19, InputOutput>>;
pub type DisplaySPI2<'d> = Display<EspSpi1InterfaceNoCS<'d>, GC9A01, PinDriver<'d, Gpio27, InputOutput>>;

pub struct ClockDisplay<'d> {
    display: DisplaySPI2<'d>,
    buffer: Box<[Rgb565]>
}

const FRAME_BUFFER_WIDTH: usize = 240;
const FRAME_BUFFER_HEIGHT: usize = 240;
const FRAME_BUFFER_SIZE: usize = FRAME_BUFFER_WIDTH * FRAME_BUFFER_HEIGHT;

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

        let spi_config = spi::config::Config::default()
            .baudrate(40_000_000.Hz())
            .write_only(true);

        let spi = SpiDeviceDriver::new(driver, Some(cs), &spi_config).unwrap();

        let di = SPIInterfaceNoCS::new(spi, dc);

        let display = Builder::gc9a01(di)
            .init(&mut delay, Some(rst))
            .map_err(|_| Box::<dyn Error>::from("display init"))
            .unwrap();

        let buffer = Self::prepare_frame_buf();

        let res = ClockDisplay {
            display,
            buffer
        };

        res
    }

    fn prepare_frame_buf() -> Box<[Rgb565]> {
        let mut v = Vec::<Rgb565>::with_capacity(FRAME_BUFFER_SIZE);
        for _ in 0..v.capacity() {
            v.push_within_capacity(Rgb565::WHITE).unwrap();
        }

        let buffer = v.into_boxed_slice();
        buffer
    }

    pub fn clear(&mut self) {
        self.display.clear(Rgb565::WHITE)
            .map_err(|_| Box::<dyn Error>::from("clear display"))
            .unwrap();
    }

    pub fn text_aligned(&mut self, text: &str, coord: Point, style: MonoTextStyle<Rgb565>, alignment: Alignment) {
        let data = self.buffer.as_mut();

        let buf: &mut [Rgb565; FRAME_BUFFER_SIZE] = data.try_into().unwrap();

        let mut fbuf = FrameBuf::new(buf, FRAME_BUFFER_WIDTH, FRAME_BUFFER_HEIGHT);

        let text = Text::with_alignment(text, coord, style, alignment);
        let bounding = text.bounding_box();

        let mut clipped = fbuf
            .clipped(&bounding);

        clipped
            .clear(Rgb565::WHITE)
            .unwrap();

        text
            .draw(&mut clipped)
            .unwrap();

        let rect = Rectangle::new(Point::zero(), fbuf.size());

        let t = data
            .into_iter()
            .map(|x| *x);

        self.display.fill_contiguous(&rect, t).unwrap();
    }

    pub fn text(&mut self, text: &str, coord: Point) {
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new(text, coord, style)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn circle(&mut self, coord: Point, diameter: u32, style: PrimitiveStyle<Rgb565>) {
        Circle::new(coord, diameter)
            .into_styled(style)
            .draw(&mut self.display).unwrap();
    }
}