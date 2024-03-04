use blinky_shared::display_interface::{ClockDisplayInterface, LayerType, RenderMode};
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
use log::{debug, info};
use mipidsi::models::GC9A01;
use mipidsi::{Builder, Display};
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use time::Instant;

pub type EspSpi1InterfaceNoCS<'d> =
    SPIInterfaceNoCS<SpiSingleDeviceDriver<'d>, PinDriver<'d, Gpio19, InputOutput>>;
pub type DisplaySPI2<'d> =
    Display<EspSpi1InterfaceNoCS<'d>, GC9A01, PinDriver<'d, Gpio27, InputOutput>>;

pub struct ClockDisplay<'a> {
    display: DisplaySPI2<'a>,
    buffer_base: Box<[Rgb565]>,
    buffer_layers: Vec<Box<[Rgb565]>>,
    static_rendered: bool,
}

impl<'a> ClockDisplayInterface for ClockDisplay<'a> {
    type Error = Infallible;
    type ColorModel = Rgb565;
    type FrameBuffer<'b> =
        FrameBuf<Self::ColorModel, &'b mut [Self::ColorModel; ClockDisplay::FRAME_BUFFER_SIZE]>;

    const FRAME_BUFFER_SIDE: usize = 240;

    const FRAME_BUFFER_SIZE: usize = Self::FRAME_BUFFER_SIDE * Self::FRAME_BUFFER_SIDE;

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
            .baudrate(80_000_000.Hz())
            .write_only(true);

        let spi = SpiDeviceDriver::new(driver, Some(cs), &spi_config).unwrap();

        let di = SPIInterfaceNoCS::new(spi, dc);

        let display = Builder::gc9a01(di)
            .with_color_order(mipidsi::ColorOrder::Bgr)
            .with_invert_colors(mipidsi::ColorInversion::Inverted)
            .init(&mut delay, Some(rst))
            .map_err(|_| Box::<dyn Error>::from("display init"))
            .unwrap();

        let buffer_layers = vec![
            Self::prepare_frame_buf(),
            Self::prepare_frame_buf(),
            Self::prepare_frame_buf(),
        ];

        let res = ClockDisplay {
            display,
            buffer_layers,
            buffer_base: Self::prepare_frame_buf(),
            static_rendered: false,
        };

        res
    }

    fn render<'c, 'd: 'c>(
        &'d mut self,
        layer: LayerType,
        mode: RenderMode,
        func: impl FnOnce(Self::FrameBuffer<'c>) -> Self::FrameBuffer<'c>,
    ) {
        if matches!(layer, LayerType::Static) && self.static_rendered {
            return;
        }

        let layer_index = layer as usize;

        let data = self.buffer_layers[layer_index].as_mut();

        let buf: &'c mut [Self::ColorModel; ClockDisplay::FRAME_BUFFER_SIZE] =
            data.try_into().unwrap();

        let mut frame = FrameBuf::new(
            buf,
            ClockDisplay::FRAME_BUFFER_SIDE,
            ClockDisplay::FRAME_BUFFER_SIDE,
        );

        let now = Instant::now();

        if matches!(mode, RenderMode::Invalidate) {
            frame.reset();
        }

        func(frame);

        if matches!(layer, LayerType::Static) {
            self.static_rendered = true;
        }

        let timing_frame = now.elapsed();

        debug!("render timing: frame {}", timing_frame);
    }

    fn commit(&mut self) {
        let now = Instant::now();

        let layers_count = self.buffer_layers.len();

        let base_layer = self.buffer_base.as_mut();

        base_layer.fill(Rgb565::BLACK);

        for layer_index in 0..layers_count {
            let layer = self.buffer_layers[layer_index].as_ref();

            for pixel_index in 0..layer.len() {
                if layer[pixel_index] == RgbColor::BLACK {
                    continue;
                }

                base_layer[pixel_index] = layer[pixel_index];
            }
        }

        let rect = Rectangle::new(
            Point::zero(),
            Size::new(
                Self::FRAME_BUFFER_SIDE as u32,
                Self::FRAME_BUFFER_SIDE as u32,
            ),
        );

        let data = self.buffer_base.as_ref();
        let iter = data.iter().map(|x| *x);

        let timing_prepare = now.elapsed();

        self.display.fill_contiguous(&rect, iter).unwrap();

        let timing_render = now.elapsed();

        debug!(
            "render commit timing: prepare {} render {}",
            timing_prepare, timing_render
        );
    }
}

impl<'a> ClockDisplay<'a> {
    fn prepare_frame_buf<TColor: RgbColor + Debug>() -> Box<[TColor]> {
        let mut v = Vec::<TColor>::with_capacity(ClockDisplay::FRAME_BUFFER_SIZE);
        for _ in 0..v.capacity() {
            v.push_within_capacity(TColor::BLACK).unwrap();
        }

        let buffer = v.into_boxed_slice();
        buffer
    }
}
