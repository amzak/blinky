use embedded_graphics::text::renderer::TextRenderer;

use crate::display_interface::ClockDisplayInterface;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::Alignment;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    prelude::{DrawTarget, *},
    primitives,
    text::Text,
};
use std::marker::PhantomData;

pub struct Graphics<TDisplay> {
    _inner: PhantomData<TDisplay>,
}

impl<TDisplay> Graphics<TDisplay>
where
    TDisplay: ClockDisplayInterface,
{
    pub fn circle(
        frame: &mut TDisplay::FrameBuffer<'_>,
        coord: Point,
        diameter: u32,
        style: PrimitiveStyle<TDisplay::ColorModel>,
    ) {
        primitives::Circle::new(coord, diameter)
            .into_styled(style)
            .draw(frame)
            .unwrap();
    }

    pub fn icon(
        frame: &mut TDisplay::FrameBuffer<'_>,
        coord: Point,
        icon: &impl ImageDrawable<Color = TDisplay::ColorModel>,
    ) {
        Image::new(icon, coord).draw(frame).unwrap();
    }

    pub fn text_aligned(
        frame: &mut TDisplay::FrameBuffer<'_>,
        text: &str,
        coord: Point,
        style: impl TextRenderer<Color = TDisplay::ColorModel>,
        alignment: Alignment,
    ) -> primitives::Rectangle {
        let text = Text::with_alignment(text, coord, style, alignment);
        let bounding_box = text.bounding_box();

        let mut clipped = frame.clipped(&bounding_box);

        let clear_color = TDisplay::ColorModel::BLACK;
        clipped.clear(clear_color).unwrap();

        text.draw(&mut clipped).unwrap();

        bounding_box
    }

    pub fn text(frame: &mut TDisplay::FrameBuffer<'_>, text: &str, coord: Point) {
        let style = MonoTextStyle::new(&FONT_6X10, TDisplay::ColorModel::WHITE);

        Text::new(text, coord, style).draw(frame).unwrap();
    }
}
