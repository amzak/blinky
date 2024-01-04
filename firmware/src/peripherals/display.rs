use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{DrawTarget, *};
use embedded_graphics::primitives::{self, PrimitiveStyle, Rectangle};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{AnyIOPin, Gpio13, Gpio14, Gpio15, Gpio19, Gpio27, InputOutput, PinDriver};
use esp_idf_hal::spi;
use esp_idf_hal::spi::config::DriverConfig;
use esp_idf_hal::spi::{Dma, SpiDeviceDriver, SpiDriver, SpiSingleDeviceDriver, SPI2};
use esp_idf_hal::units::FromValueType;
use mipidsi::models::GC9A01;
use mipidsi::{Builder, Display};
use std::error::Error;

pub type EspSpi1InterfaceNoCS<'d> =
    SPIInterfaceNoCS<SpiSingleDeviceDriver<'d>, PinDriver<'d, Gpio19, InputOutput>>;
pub type DisplaySPI2<'d> =
    Display<EspSpi1InterfaceNoCS<'d>, GC9A01, PinDriver<'d, Gpio27, InputOutput>>;

pub type FrameBuffer<'d> = FrameBuf<Rgb565, &'d mut [Rgb565; 57600]>;

pub struct ClockDisplay<'d> {
    display: DisplaySPI2<'d>,
    buffer: Box<[Rgb565]>,
}

impl<'d> ClockDisplay<'d> {
}

pub trait ClockDisplayInterface {
    const FRAME_BUFFER_WIDTH: usize = 240;
    const FRAME_BUFFER_HEIGHT: usize = 240;
    const FRAME_BUFFER_SIZE: usize = Self::FRAME_BUFFER_WIDTH * Self::FRAME_BUFFER_HEIGHT;

    fn create() -> Self;
    fn render(&mut self, func: impl Fn(&mut FrameBuffer));
}

impl<'d> ClockDisplayInterface for ClockDisplay<'d> {
    fn create() -> Self {
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
            .with_color_order(mipidsi::ColorOrder::Bgr)
            .with_invert_colors(mipidsi::ColorInversion::Inverted)
            .init(&mut delay, Some(rst))
            .map_err(|_| Box::<dyn Error>::from("display init"))
            .unwrap();

        let buffer = Self::prepare_frame_buf();

        let res = ClockDisplay { display, buffer };

        res
    }

    fn render(&mut self, func: impl Fn(&mut FrameBuffer)) {
        let data = self.buffer.as_mut();

        let buf: &mut [Rgb565; ClockDisplayInterface::FRAME_BUFFER_SIZE] = data.try_into().unwrap();

        let mut frame: FrameBuffer = FrameBuf::new(
            buf,
            ClockDisplayInterface::FRAME_BUFFER_WIDTH,
            ClockDisplayInterface::FRAME_BUFFER_HEIGHT,
        );

        func(&mut frame);

        let rect = Rectangle::new(Point::zero(), frame.size());

        let t = data.into_iter().map(|x| *x);

        self.display.fill_contiguous(&rect, t).unwrap();
    }
}

impl<'d> ClockDisplay<'d> {
    fn prepare_frame_buf() -> Box<[Rgb565]> {
        let mut v = Vec::<Rgb565>::with_capacity(ClockDisplayInterface::FRAME_BUFFER_SIZE);
        for _ in 0..v.capacity() {
            v.push_within_capacity(Rgb565::BLACK).unwrap();
        }

        let buffer = v.into_boxed_slice();
        buffer
    }
}

impl Drop for ClockDisplay<'_> {
    fn drop(&mut self) {
        self.render(|frame| {
            let style = PrimitiveStyle::with_fill(Rgb565::BLACK);

            primitives::Circle::new(Point::zero(), ClockDisplayInterface::FRAME_BUFFER_WIDTH as u32)
                .into_styled(style)
                .draw(frame)
                .unwrap();
        });
    }
}
