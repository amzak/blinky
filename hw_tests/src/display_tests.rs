use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::spi::Operation;
use esp_idf_hal::units::FromValueType;
use esp_idf_hal::{
    delay::Ets,
    spi::{self, config::DriverConfig, Dma, SpiDeviceDriver, SpiDriver},
};
use peripherals::pins::mapping::PinsMapping;

#[repr(u8)]
pub enum SH8601Commands {
    SH8601_C_NOP = 0x00,          // nop
    SH8601_C_SWRESET = 0x01,      // Software Reset
    SH8601_R_RDID = 0x04,         // Read Display Identification Information ID/1/2/3
    SH8601_R_RDNERRORSDSI = 0x05, // Read Number of Errors on DSI
    SH8601_R_RDPOWERMODE = 0x0A,  // Read Display Power Mode
    SH8601_R_RDMADCTL = 0x0B,     // Read Display MADCTL
    SH8601_R_RDPIXFMT = 0x0C,     // Read Display Pixel Format
    SH8601_R_RDIMGFMT = 0x0D,     // Read Display Image Mode
    SH8601_R_RDSIGMODE = 0x0E,    // Read Display Signal Mode
    SH8601_R_RDSELFDIAG = 0x0F,   // Read Display Self-Diagnostic Result

    SH8601_C_SLPIN = 0x10,  // Sleep In
    SH8601_C_SLPOUT = 0x11, // Sleep Out
    SH8601_C_PTLON = 0x12,  // Partial Display On
    SH8601_C_NORON = 0x13,  // Normal Display mode on

    SH8601_C_INVOFF = 0x20,  // Inversion Off
    SH8601_C_INVON = 0x21,   // Inversion On
    SH8601_C_ALLPOFF = 0x22, // All pixels off
    SH8601_C_ALLPON = 0x23,  // All pixels on
    SH8601_C_DISPOFF = 0x28, // Display off
    SH8601_C_DISPON = 0x29,  // Display on
    SH8601_W_CASET = 0x2A,   // Column Address Set
    SH8601_W_PASET = 0x2B,   // Page Address Set
    SH8601_W_RAMWR = 0x2C,   // Memory Write Start

    SH8601_W_PTLAR = 0x30,   // Partial Area Row Set
    SH8601_W_PTLAC = 0x31,   // Partial Area Column Set
    SH8601_C_TEAROFF = 0x34, // Tearing effect off
    SH8601_WC_TEARON = 0x35, // Tearing effect on
    SH8601_W_MADCTL = 0x36,  // Memory data access control
    SH8601_C_IDLEOFF = 0x38, // Idle Mode Off
    SH8601_C_IDLEON = 0x39,  // Idle Mode On
    SH8601_W_PIXFMT = 0x3A,  // Write Display Pixel Format
    SH8601_W_WRMC = 0x3C,    // Memory Write Continue

    SH8601_W_SETTSL = 0x44,             // Write Tearing Effect Scan Line
    SH8601_R_GETSL = 0x45,              // Read Scan Line Number
    SH8601_C_SPIROFF = 0x46,            // SPI read Off
    SH8601_C_SPIRON = 0x47,             // SPI read On
    SH8601_C_AODMOFF = 0x48,            // AOD Mode Off
    SH8601_C_AODMON = 0x49,             // AOD Mode On
    SH8601_W_WDBRIGHTNESSVALAOD = 0x4A, // Write Display Brightness Value in AOD Mode
    SH8601_R_RDBRIGHTNESSVALAOD = 0x4B, // Read Display Brightness Value in AOD Mode
    SH8601_W_DEEPSTMODE = 0x4F,         // Deep Standby Mode On

    SH8601_W_WDBRIGHTNESSVALNOR = 0x51, // Write Display Brightness Value in Normal Mode
    SH8601_R_RDBRIGHTNESSVALNOR = 0x52, // Read display brightness value in Normal Mode
    SH8601_W_WCTRLD1 = 0x53,            // Write CTRL Display1
    SH8601_R_RCTRLD1 = 0x54,            // Read CTRL Display1
    SH8601_W_WCTRLD2 = 0x55,            // Write CTRL Display2
    SH8601_R_RCTRLD2 = 0x56,            // Read CTRL Display2
    SH8601_W_WCE = 0x58,                // Write CE
    SH8601_R_RCE = 0x59,                // Read CE

    SH8601_W_WDBRIGHTNESSVALHBM = 0x63, // Write Display Brightness Value in HBM Mode
    SH8601_R_WDBRIGHTNESSVALHBM = 0x64, // Read Display Brightness Value in HBM Mode
    SH8601_W_WHBMCTL = 0x66,            // Write HBM Control

    SH8601_W_COLORSET0 = 0x70,  // Color Set 0
    SH8601_W_COLORSET1 = 0x71,  // Color Set 1
    SH8601_W_COLORSET2 = 0x72,  // Color Set 2
    SH8601_W_COLORSET3 = 0x73,  // Color Set 3
    SH8601_W_COLORSET4 = 0x74,  // Color Set 4
    SH8601_W_COLORSET5 = 0x75,  // Color Set 5
    SH8601_W_COLORSET6 = 0x76,  // Color Set 6
    SH8601_W_COLORSET7 = 0x77,  // Color Set 7
    SH8601_W_COLORSET8 = 0x78,  // Color Set 8
    SH8601_W_COLORSET9 = 0x79,  // Color Set 9
    SH8601_W_COLORSET10 = 0x7A, // Color Set 10
    SH8601_W_COLORSET11 = 0x7B, // Color Set 11
    SH8601_W_COLORSET12 = 0x7C, // Color Set 12
    SH8601_W_COLORSET13 = 0x7D, // Color Set 13
    SH8601_W_COLORSET14 = 0x7E, // Color Set 14
    SH8601_W_COLORSET15 = 0x7F, // Color Set 15

    SH8601_W_COLOROPTION = 0x80, // Color Option

    SH8601_R_RDDBSTART = 0xA1,         // Read DDB start
    SH8601_R_DDBCONTINUE = 0xA8,       // Read DDB Continue
    SH8601_R_RFIRCHECKSUN = 0xAA,      // Read First Checksum
    SH8601_R_RCONTINUECHECKSUN = 0xAF, // Read Continue Checksum

    SH8601_W_SPIMODECTL = 0xC4, // SPI mode control

    SH8601_R_RDID1 = 0xDA, // Read ID1
    SH8601_R_RDID2 = 0xDB, // Read ID2
    SH8601_R_RDID3 = 0xDC, // Read ID3

    SH8601_W_FE = 0xFE,
}

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

    log::info!("resetting display...");

    // reset
    rst.set_high().unwrap();
    delay.delay_ms(10);
    rst.set_low().unwrap();
    delay.delay_ms(200);
    rst.set_high().unwrap();
    delay.delay_ms(200);

    init_display(&mut spi, &mut delay);
}

fn command<'a>(spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>, cmd: SH8601Commands) {
    spi.transaction(&mut [Operation::WriteWithWidth(
        &[0x02, 0x00, cmd as u8, 0x00],
        spi::config::LineWidth::Single,
    )])
    .unwrap();
}

fn command_wd<'a>(spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>, cmd: SH8601Commands, data: &[u8]) {
    spi.transaction(&mut [
        Operation::WriteWithWidth(
            &[0x02, 0x00, cmd as u8, 0x00],
            spi::config::LineWidth::Single,
        ),
        Operation::WriteWithWidth(data, spi::config::LineWidth::Single),
    ])
    .unwrap();
}

fn pixels<'a>(spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>, data: &[u8]) {
    spi.transaction(&mut [
        Operation::WriteWithWidth(&[0x32, 0x00, 0x2c, 0x00], spi::config::LineWidth::Single),
        Operation::WriteWithWidth(data, spi::config::LineWidth::Quad),
    ])
    .unwrap();
}

fn init_display<'a, DELAY: embedded_hal::delay::DelayNs>(
    spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>,
    delay: &mut DELAY,
) {
    let mut buffer = [0u8; 4];

    command(spi, SH8601Commands::SH8601_C_SLPOUT);

    delay.delay_us(120_000);

    command(spi, SH8601Commands::SH8601_C_NORON);
    //command(spi, SH8601Commands::SH8601_C_INVOFF);

    // command_wd(spi, SH8601Commands::SH8601_W_FE, &[0x00]);

    // command_wd(spi, SH8601Commands::SH8601_W_SPIMODECTL, &[0x0]);

    //let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());

    // let options = ModelOptions::with_all((480, 480), (0, 0));
    // let madctl = SetAddressMode::from(&options);

    // let mut bytes = [0u8];
    // madctl.fill_params_buf(&mut bytes);

    //command_wd(spi, SH8601Commands::SH8601_W_MADCTL, &bytes);

    //di.write_command(SetPixelFormat::new(pf))?;
    command_wd(spi, SH8601Commands::SH8601_W_PIXFMT, &[0x05]);

    command(spi, SH8601Commands::SH8601_C_DISPON);

    command_wd(spi, SH8601Commands::SH8601_W_WCTRLD1, &[0x28]);

    command_wd(spi, SH8601Commands::SH8601_W_WDBRIGHTNESSVALNOR, &[0x00]);

    //command_wd(spi, SH8601Commands::SH8601_W_WDBRIGHTNESSVALNOR, &[0x00]);

    command_wd(spi, SH8601Commands::SH8601_W_WCE, &[0x00]);

    command(spi, SH8601Commands::SH8601_C_INVOFF);

    delay.delay_us(100_000);

    command_wd(spi, SH8601Commands::SH8601_W_WDBRIGHTNESSVALNOR, &[0x55]);

    const width: usize = 480;
    const height: usize = 480;

    let mut data = vec![0x3c; width * height * 2];

    let start_column: u16 = 0;
    let end_column: u16 = 49;

    let start_row: u16 = 200;
    let end_row: u16 = 200;

    let region_size: usize = (end_column as usize - start_column as usize + 1)
        * (end_row as usize - start_row as usize + 1)
        * 2;

    buffer[0..2].copy_from_slice(&start_column.to_be_bytes());
    buffer[2..4].copy_from_slice(&end_column.to_be_bytes());

    command_wd(spi, SH8601Commands::SH8601_W_CASET, &buffer);

    log::info!("CASET: {:02X?}", &buffer);

    buffer[0..2].copy_from_slice(&start_row.to_be_bytes());
    buffer[2..4].copy_from_slice(&end_row.to_be_bytes());

    command_wd(spi, SH8601Commands::SH8601_W_PASET, &buffer);

    log::info!("PASET: {:02X?}", &buffer);

    let range = 0..region_size;

    log::info!("data length: {}", data[range.clone()].len());

    pixels(spi, &data[range]);

    delay.delay_ms(10000);
}
