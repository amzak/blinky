use std::collections::HashMap;

use embedded_graphics::{
    geometry::Point,
    image::{Image, ImageDrawable, ImageDrawableExt, ImageRawBE, SubImage},
    pixelcolor::{raw::RawU16, PixelColor, RgbColor},
    Drawable,
};
use embedded_graphics_framebuf::FrameBuf;

use crate::{calendar::CalendarEventIcon, error::Error};

struct IconsRegistry<
    'a,
    TColor: RgbColor + From<RawU16> + From<<TColor as PixelColor>::Raw>,
    TImage: FrameBuf,
> {
    buffer: HashMap<CalendarEventIcon, Image<'a, TImage>>,
}

impl<'a, TColor, TImage> IconsRegistry<'a, TColor, TImage>
where
    TColor: RgbColor + From<RawU16> + From<<TColor as PixelColor>::Raw>,
    TImage: ImageDrawable<Color = TColor>,
{
    pub fn new() -> Self {
        let mut map = HashMap::new();

        Self::register_icon(&mut map, icon_type, small_mdi_icons::TrainVariant::new(color))

        Self { image_map: map }
    }

    fn register_icon<TIcon>(
        map: &mut HashMap<CalendarEventIcon, Image<'a, TImage>>,
        icon_type: CalendarEventIcon,
        icon: TImage,
    ) -> Result<(), Error>
    where
        TIcon: ImageDrawable<Color = TColor>,
    {
        let image = Image::new(&icon, Point::zero());

        map[&icon_type] = image;
        Ok(())
    }
}
