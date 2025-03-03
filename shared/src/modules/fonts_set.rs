use u8g2_fonts::{fonts, Font};

pub trait FontSet {
    fn get_clock_font() -> impl Font;

    fn get_day_font() -> impl Font;

    fn get_temperature_font() -> impl Font;

    fn get_event_details_font() -> impl Font;
}

pub struct FontSet240 {}

impl FontSet240 {
    pub fn new() -> FontSet240 {
        FontSet240 {}
    }
}

impl FontSet for FontSet240 {
    fn get_clock_font() -> impl Font {
        fonts::u8g2_font_spleen16x32_mn
    }

    fn get_day_font() -> impl Font {
        fonts::u8g2_font_wqy16_t_gb2312b
    }

    fn get_temperature_font() -> impl Font {
        fonts::u8g2_font_siji_t_6x10
    }

    fn get_event_details_font() -> impl Font {
        fonts::u8g2_font_siji_t_6x10
    }
}

pub struct FontSet466 {}

impl FontSet466 {
    pub fn new() -> FontSet466 {
        FontSet466 {}
    }
}

impl FontSet for FontSet466 {
    fn get_clock_font() -> impl Font {
        fonts::u8g2_font_spleen32x64_mn
    }

    fn get_day_font() -> impl Font {
        fonts::u8g2_font_spleen32x64_mn
    }

    fn get_temperature_font() -> impl Font {
        fonts::u8g2_font_spleen12x24_mf
    }

    fn get_event_details_font() -> impl Font {
        fonts::u8g2_font_spleen12x24_mf
    }
}
