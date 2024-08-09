use blinky_shared::display_interface::{ClockDisplayInterface, LayerType, RenderMode};
use display_interface_spi::SPIInterface;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{DrawTarget, *};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_framebuf::FrameBuf;
use enumflags2::{BitFlag, BitFlags};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{AnyIOPin, Gpio13, Gpio14, Gpio15, Gpio19, Gpio27, InputOutput, PinDriver};
use esp_idf_hal::spi;
use esp_idf_hal::spi::config::DriverConfig;
use esp_idf_hal::spi::{Dma, SpiDeviceDriver, SpiDriver, SpiSingleDeviceDriver, SPI2};
use esp_idf_hal::units::FromValueType;
use log::info;
use mipidsi::models::Model;
use mipidsi::options::{ColorInversion, ColorOrder};
use mipidsi::{Builder, Display};
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use time::Instant;

use crate::peripherals::GC9A01_NOINIT::Gc9a01Noinit;

pub type EspSpi1InterfaceNoCS<'d> =
    SPIInterface<SpiSingleDeviceDriver<'d>, PinDriver<'d, Gpio19, InputOutput>>;
pub type DisplaySPI2<'d> = Display<
    EspSpi1InterfaceNoCS<'d>,
    impl Model<ColorFormat = Rgb565>,
    PinDriver<'d, Gpio27, InputOutput>,
>;

pub struct ClockDisplay<'a> {
    display: DisplaySPI2<'a>,
    buffer_base: Box<[Rgb565]>,
    buffer_layers: Vec<Box<[Rgb565]>>,
    static_rendered: bool,
    is_first_render: bool,
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
            .polling(true)
            .write_only(true);

        let spi = SpiDeviceDriver::new(driver, Some(cs), &spi_config).unwrap();

        let di = SPIInterface::new(spi, dc);

        info!("initializing display spi...");

        let display = Builder::new(Gc9a01Noinit::new(false), di)
            .color_order(ColorOrder::Bgr)
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(rst)
            .init(&mut delay)
            .map_err(|_| Box::<dyn Error>::from("display init"))
            .unwrap();

        info!("display spi initialized");

        let buffer_layers = vec![
            Self::prepare_frame_buf(),
            Self::prepare_frame_buf(),
            Self::prepare_frame_buf(),
        ];

        info!("display buffers initialized");

        let res = ClockDisplay {
            display,
            buffer_layers,
            buffer_base: Self::prepare_frame_buf(),
            static_rendered: false,
            is_first_render: true,
        };

        res
    }

    fn render<'c, 'd: 'c>(
        &'d mut self,
        layer: LayerType,
        mode: RenderMode,
        func: impl FnOnce(Self::FrameBuffer<'c>) -> Self::FrameBuffer<'c>,
    ) {
        let layer_value = layer as i32;

        let log2 = f32::log2(layer_value as f32);

        let layer_index: usize = f32::round(log2) as usize;

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

        let timing_reset = now.elapsed();

        func(frame);

        let timing_frame = now.elapsed() - timing_reset;

        info!(
            "render timing: layer {:?} reset {} frame {}",
            layer, timing_reset, timing_frame
        );
    }

    fn commit(&mut self, layers_mask: BitFlags<LayerType>) {
        let now = Instant::now();

        let layers_count = self.buffer_layers.len();

        let base_layer = self.buffer_base.as_mut();

        base_layer.fill(Rgb565::BLACK);

        let timing_fill_black = now.elapsed();

        for layer_index in 0..layers_count {
            if !layers_mask.contains(BitFlags::from_bits_truncate(1 << layer_index)) {
                continue;
            }

            let layer = self.buffer_layers[layer_index].as_ref();

            for pixel_index in 0..layer.len() {
                if layer[pixel_index] == RgbColor::BLACK {
                    continue;
                }

                base_layer[pixel_index] = layer[pixel_index];
            }
        }

        let timing_merge = now.elapsed() - timing_fill_black;

        let rect = Rectangle::new(
            Point::zero(),
            Size::new(
                Self::FRAME_BUFFER_SIDE as u32,
                Self::FRAME_BUFFER_SIDE as u32,
            ),
        );

        let data = self.buffer_base.as_ref();

        //self.is_first_render = false;

        if self.is_first_render {
            let iter = data.iter().enumerate().filter_map(|item| {
                let color = *item.1;
                if color == Self::ColorModel::BLACK {
                    return None;
                }

                let y = item.0 / Self::FRAME_BUFFER_SIDE;
                let x = item.0 - y * Self::FRAME_BUFFER_SIDE;
                let point = Point::new(x as i32, y as i32);

                Some(Pixel(point, color))
            });

            self.display.draw_iter(iter).unwrap();

            self.is_first_render = false;
        } else {
            let iter = data.iter().map(|x| *x);

            self.display.fill_contiguous(&rect, iter).unwrap();
        }

        let timing_render = now.elapsed() - timing_fill_black - timing_merge;

        info!(
            "render commit timing: clear_black {} merge {}  render {}",
            timing_fill_black, timing_merge, timing_render
        );
    }
}

impl<'a> Drop for ClockDisplay<'a> {
    fn drop(&mut self) {
        let mut delay = Ets;

        self.display.sleep(&mut delay).unwrap();

        info!("display set to sleep mode");
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
