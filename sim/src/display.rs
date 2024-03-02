use std::convert::Infallible;

use blinky_shared::display_interface::ClockDisplayInterface;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use std::fmt::Debug;

pub struct SimDisplay {
    buffer: Box<[Rgb565]>,
    display: SimulatorDisplay<Rgb565>,
    window: Window,
}

impl ClockDisplayInterface for SimDisplay {
    type Error = Infallible;

    type ColorModel = Rgb565;

    type FrameBuffer<'b> =
        FrameBuf<Self::ColorModel, &'b mut [Self::ColorModel; Self::FRAME_BUFFER_SIZE]>;

    const FRAME_BUFFER_SIDE: usize = 240;

    const FRAME_BUFFER_SIZE: usize = Self::FRAME_BUFFER_SIDE * Self::FRAME_BUFFER_SIDE;

    fn create() -> Self {
        let buffer = Self::prepare_frame_buf();
        let display = SimulatorDisplay::<Rgb565>::new(Size::new(
            Self::FRAME_BUFFER_SIDE as u32,
            Self::FRAME_BUFFER_SIDE as u32,
        ));

        let output_settings = OutputSettingsBuilder::new()
            .scale(3)
            .pixel_spacing(1)
            //.theme(BinaryColorTheme::OledBlue)
            .build();

        let window = Window::new("Hello World", &output_settings);

        SimDisplay {
            buffer,
            display,
            window,
        }
    }

    fn render<'b, 'a: 'b>(
        &'a mut self,
        func: impl FnOnce(Self::FrameBuffer<'b>) -> Self::FrameBuffer<'b>,
    ) {
        let data = self.buffer.as_mut();

        let buf: &'b mut [Self::ColorModel; Self::FRAME_BUFFER_SIZE] = data.try_into().unwrap();

        let mut frame = FrameBuf::new(buf, Self::FRAME_BUFFER_SIDE, Self::FRAME_BUFFER_SIDE);

        //frame.clear(Rgb565::BLACK).unwrap();
        frame = func(frame);

        let t = frame.into_iter();

        self.display.draw_iter(t).unwrap();

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
