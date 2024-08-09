use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{raw::RawU16, PixelColor, Rgb555, RgbColor},
};
use enumflags2::{bitflags, BitFlags};
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[bitflags]
pub enum LayerType {
    Static = 1 << 0,
    Clock = 1 << 1,
    Events = 1 << 2,
}

pub enum RenderMode {
    Invalidate,
    Ammend,
}

pub trait ClockDisplayInterface {
    type Error: Debug;

    type ColorModel: RgbColor
        + From<RawU16>
        + From<Rgb555>
        + From<<Self::ColorModel as PixelColor>::Raw>
        + PixelColor<Raw = RawU16>
        + Default;

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

    fn commit(&mut self, layers_mask: BitFlags<LayerType>);
}
