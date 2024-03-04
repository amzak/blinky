use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{raw::RawU16, RgbColor},
};
use std::fmt::Debug;

#[derive(Clone, Copy)]
pub enum LayerType {
    Static = 0,
    Clock = 1,
    Events = 2,
}

pub enum RenderMode {
    Invalidate,
    Ammend,
}

pub trait ClockDisplayInterface {
    type Error: Debug;
    type ColorModel: RgbColor + From<RawU16>;
    type FrameBuffer<'b>: DrawTarget<Error = Self::Error, Color = Self::ColorModel>;

    const FRAME_BUFFER_SIDE: usize;
    const FRAME_BUFFER_SIZE: usize;

    fn create() -> Self;

    fn render<'b, 'a: 'b>(
        &'a mut self,
        layer: LayerType,
        mode: RenderMode,
        func: impl FnOnce(Self::FrameBuffer<'b>) -> Self::FrameBuffer<'b>,
    );

    fn commit(&mut self);
}
