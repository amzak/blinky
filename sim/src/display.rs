use std::{convert::Infallible, time::Instant};

use blinky_shared::display_interface::{ClockDisplayInterface, LayerType, RenderMode};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    primitives::Rectangle,
    Pixel,
};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use enumflags2::{BitFlag, BitFlags};
use log::{debug, info};

use std::fmt::Debug;

pub struct SimDisplay {
    display: SimulatorDisplay<Rgb565>,
    buffer_base: Box<[Rgb565]>,
    buffer_layers: Vec<Box<[Rgb565]>>,
    is_first_render: bool,
    window: Window,
}

unsafe impl Send for SimDisplay {}

impl SimDisplay {
    pub fn create() -> Self {
        let display = SimulatorDisplay::<Rgb565>::new(Size::new(
            Self::FRAME_BUFFER_SIDE as u32,
            Self::FRAME_BUFFER_SIDE as u32,
        ));

        let output_settings = OutputSettingsBuilder::new()
            .scale(1)
            .pixel_spacing(1)
            .max_fps(5)
            .build();

        let window = Window::new("Blinky watch sim", &output_settings);

        let buffer_layers = vec![
            Self::prepare_frame_buf(),
            Self::prepare_frame_buf(),
            Self::prepare_frame_buf(),
        ];

        SimDisplay {
            display,
            buffer_layers,
            buffer_base: Self::prepare_frame_buf(),
            is_first_render: true,
            window,
        }
    }
}

impl ClockDisplayInterface for SimDisplay {
    type Error = Infallible;

    type ColorModel = Rgb565;

    type FrameBuffer<'b> =
        FrameBuf<Self::ColorModel, &'b mut [Self::ColorModel; Self::FRAME_BUFFER_SIZE]>;

    const FRAME_BUFFER_SIDE: usize = 466;

    const FRAME_BUFFER_SIZE: usize = Self::FRAME_BUFFER_SIDE * Self::FRAME_BUFFER_SIDE;

    fn render<'c, 'd: 'c>(
        &'d mut self,
        layer: LayerType,
        mode: RenderMode,
        func: impl FnOnce(Self::FrameBuffer<'c>) -> Self::FrameBuffer<'c>,
    ) {
        let layer_value = layer as i32;

        let log2 = f32::log2(layer_value as f32);

        let layer_index: usize = f32::round(log2) as usize;

        debug!("rendering {:?} to index {:?}", layer, layer_index);

        let data = self.buffer_layers[layer_index].as_mut();

        let buf: &'c mut [Self::ColorModel; Self::FRAME_BUFFER_SIZE] = data.try_into().unwrap();

        let mut frame = FrameBuf::new(buf, Self::FRAME_BUFFER_SIDE, Self::FRAME_BUFFER_SIDE);

        let now = Instant::now();

        if matches!(mode, RenderMode::Invalidate) {
            frame.reset();
        }

        func(frame);

        let timing_frame = now.elapsed();

        debug!("render timing: frame {}", timing_frame.as_millis());
    }

    fn commit(&mut self, merge_layers_mask: BitFlags<LayerType>) {
        let layers_count = self.buffer_layers.len();

        let base_layer = self.buffer_base.as_mut();

        base_layer.fill(Rgb565::BLACK);

        for layer_index in 0..layers_count {
            if !merge_layers_mask.contains(BitFlags::from_bits_truncate(1 << layer_index)) {
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

        let rect = Rectangle::new(
            Point::zero(),
            Size::new(
                Self::FRAME_BUFFER_SIDE as u32,
                Self::FRAME_BUFFER_SIDE as u32,
            ),
        );

        let data = self.buffer_base.as_mut();

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

        self.window.update(&self.display);
    }
}

impl SimDisplay {
    fn prepare_frame_buf<TColor: RgbColor + Debug>() -> Box<[TColor]> {
        let mut v = Vec::<TColor>::with_capacity(Self::FRAME_BUFFER_SIZE);
        for _ in 0..v.capacity() {
            v.push_within_capacity(TColor::BLACK).unwrap();
        }

        let buffer = v.into_boxed_slice();
        buffer
    }
}
