use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use tokio::sync::broadcast;
use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

extern crate blinky_shared;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(240, 240));

    let line_style = PrimitiveStyle::with_stroke(Rgb565::WHITE, 1);
    let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::WHITE);

    let (commands_sender, _) = broadcast::channel::<Commands>(32);
    let (events_sender, _) = broadcast::channel::<Events>(32);

    //Renderer::start::<>()

    Circle::new(Point::new(72, 8), 48)
        .into_styled(line_style)
        .draw(&mut display)?;

    Line::new(Point::new(48, 16), Point::new(8, 16))
        .into_styled(line_style)
        .draw(&mut display)?;

    Line::new(Point::new(48, 16), Point::new(64, 32))
        .into_styled(line_style)
        .draw(&mut display)?;

    Rectangle::new(Point::new(79, 15), Size::new(34, 34))
        .into_styled(line_style)
        .draw(&mut display)?;

    Text::new("Hello World!", Point::new(5, 5), text_style).draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        //.theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello World", &output_settings).show_static(&display);

    Ok(())
}
