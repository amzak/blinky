use embedded_graphics::text::renderer::TextRenderer;

use crate::display_interface::ClockDisplayInterface;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::Alignment;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    prelude::*,
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
        top_left: Point,
        diameter: u32,
        style: PrimitiveStyle<TDisplay::ColorModel>,
    ) {
        primitives::Circle::new(top_left, diameter)
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

    pub fn icon_center(
        frame: &mut TDisplay::FrameBuffer<'_>,
        coord: Point,
        icon: &impl ImageDrawable<Color = TDisplay::ColorModel>,
    ) {
        Image::with_center(icon, coord).draw(frame).unwrap();
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

        text.draw(&mut clipped).unwrap();

        bounding_box
    }

    pub fn text(frame: &mut TDisplay::FrameBuffer<'_>, text: &str, coord: Point) {
        let style = MonoTextStyle::new(&FONT_6X10, TDisplay::ColorModel::WHITE);

        Text::new(text, coord, style).draw(frame).unwrap();
    }

    pub fn helix(
        frame: &mut TDisplay::FrameBuffer<'_>,
        bounding_box: Rectangle,
        step: u32,
        num_revolutions: u8,
    ) {
        let num_of_arcs: u32 = num_revolutions as u32 * 2;
        let mut top_left = bounding_box.top_left;
        let mut height = bounding_box.size.height;
        let mut angle = Angle::from_degrees(-90.0);
        let half_turn = Angle::from_degrees(180.0);

        let mut outer_circle = primitives::Circle::new(top_left, height);

        let mut counter = 0;
        while counter < num_of_arcs {
            let arc = primitives::Arc::from_circle(outer_circle, angle, half_turn);

            let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, 1);
            arc.draw_styled(&style, frame).unwrap();

            if counter % 2 == 0 {
                top_left += Point::new(step as i32, step as i32);
            } else {
            }

            height -= step as u32;
            outer_circle = primitives::Circle::new(top_left, height);

            counter += 1;
            angle += half_turn;
        }
    }
}
