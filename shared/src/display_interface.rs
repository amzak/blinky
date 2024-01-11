use embedded_graphics::{draw_target::DrawTarget, pixelcolor::RgbColor};
use std::fmt::Debug;

pub trait ClockDisplayInterface {
    type Error: Debug;
    type ColorModel: RgbColor;
    type FrameBuffer<'b>: DrawTarget<Error = Self::Error, Color = Self::ColorModel>;

    const FRAME_BUFFER_SIDE: usize;
    const FRAME_BUFFER_SIZE: usize;

    fn create() -> Self;

    fn render<'b, 'a: 'b>(
        &'a mut self,
        func: impl FnOnce(Self::FrameBuffer<'b>) -> Self::FrameBuffer<'b>,
    );
}
