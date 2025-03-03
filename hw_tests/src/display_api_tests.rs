use crate::spi_interface_no_dc::SpiInterfaceNoDC;
use crate::SH8601::SH8601;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTargetExt, Point, RgbColor, Size};
use embedded_graphics::primitives::{self, PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_hal::delay::DelayNs;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::units::FromValueType;
use esp_idf_hal::{
    delay::Ets,
    spi::{self, config::DriverConfig, Dma, SpiDeviceDriver, SpiDriver},
};
use log::info;
use mipidsi::options::ColorOrder;
use mipidsi::Builder;
use peripherals::pins::mapping::PinsMapping;
use std::error::Error;
use u8g2_fonts::{fonts, U8g2TextStyle};

const FRAME_BUFFER_SIDE: usize = 466;
const FRAME_BUFFER_SIZE: usize = FRAME_BUFFER_SIDE * FRAME_BUFFER_SIDE;

pub(crate) fn run(spi: esp_idf_hal::spi::SPI2, pins_mapping: &mut impl PinsMapping) {
    let mut delay = Ets;

    let cs = pins_mapping.get_spi_cs_pin();
    let sclk = pins_mapping.get_spi_sclk_pin();
    let sdo = pins_mapping.get_spi_sdo_pin();

    let sdo1_pin = pins_mapping.get_spi_sdo1_pin();
    let sdo2_pin = pins_mapping.get_spi_sdo2_pin();
    let sdo3_pin = pins_mapping.get_spi_sdo3_pin();

    let mut rst = pins_mapping.get_display_rst_pin();

    let config = DriverConfig {
        dma: Dma::Disabled,
        intr_flags: Default::default(),
    };

    let driver =
        SpiDriver::new_quad(spi, sclk, sdo, sdo1_pin, sdo2_pin, sdo3_pin, &config).unwrap();

    let spi_config = spi::config::Config::default()
        .duplex(spi::config::Duplex::Half)
        .baudrate(40_000_000.Hz())
        .write_only(true)
        .polling(true);

    let mut spi = SpiDeviceDriver::new(driver, Some(cs), &spi_config).unwrap();

    let en_pin = pins_mapping.get_display_en_pin();
    let mut pin_driver = PinDriver::output(en_pin).unwrap();
    pin_driver.set_high().unwrap();

    delay.delay_ms(100);

    let mut buffer = [0u8; 256];

    let di = SpiInterfaceNoDC::new(spi, &mut buffer);

    info!("initializing display spi...");

    let mut display = Builder::new(SH8601::new(), di)
        .display_size(FRAME_BUFFER_SIDE as u16, FRAME_BUFFER_SIDE as u16)
        .color_order(ColorOrder::Rgb)
        .reset_pin(rst)
        .init(&mut delay)
        .map_err(|_| Box::<dyn Error>::from("display init"))
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let mut solid_style = PrimitiveStyle::with_stroke(Rgb565::WHITE, 1);
    solid_style.fill_color = Some(Rgb565::BLACK);

    let point = Point::new(100, 100);

    primitives::Circle::with_center(point, 50)
        .draw_styled(&solid_style, &mut display)
        .unwrap();

    let style = U8g2TextStyle::new(fonts::u8g2_font_spleen16x32_mn, Rgb565::WHITE);

    let text = Text::with_alignment(
        "12:00:00",
        Point::new(233, 233),
        style,
        embedded_graphics::text::Alignment::Center,
    );

    let bounding_box = text.bounding_box();
    let mut clipped = display.clipped(&bounding_box);

    text.draw(&mut clipped).unwrap();

    delay.delay_ms(1000);

    // let mut max = 0;
    // let iter = (0..10000).map(|i| {
    //     if max < i {
    //         max = i;
    //     }

    //     if i % 2 == 0 {
    //         Rgb565::WHITE
    //     } else {
    //         Rgb565::BLACK
    //     }
    // });

    // let rect = Rectangle::new(Point::zero(), Size::new_equal(100));

    // display.fill_contiguous(&rect, iter).unwrap();

    let a = 250;
    let b = 250;

    let length: usize = a * b;

    display
        .set_pixels(
            100,
            100,
            100 + a as u16 - 1,
            100 + b as u16 - 1,
            core::iter::repeat_n(Rgb565::RED, length),
        )
        .unwrap();

    delay.delay_ms(10000);

    info!("display spi initialized");
}
