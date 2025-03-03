use embedded_graphics::pixelcolor::Rgb565;
use mipidsi::models::Model;

use mipidsi::dcs::{
    BitsPerPixel, EnterNormalMode, ExitSleepMode, InterfaceExt, PixelFormat, SetAddressMode,
    SetDisplayOn, SetInvertMode, SetPixelFormat,
};

pub struct SH8601 {}

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
}

impl SH8601 {
    pub fn new() -> Self {
        SH8601 {}
    }
}

impl Model for SH8601 {
    type ColorFormat = Rgb565;

    const FRAMEBUFFER_SIZE: (u16, u16) = (466, 466);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &mipidsi::options::ModelOptions,
    ) -> Result<mipidsi::dcs::SetAddressMode, DI::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
        DI: mipidsi::interface::Interface,
    {
        delay.delay_us(120_000);

        di.write_raw(SH8601Commands::SH8601_C_SLPOUT as u8, &[])?;

        delay.delay_us(120_000);

        di.write_raw(SH8601Commands::SH8601_C_NORON as u8, &[])?;

        //let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        //di.write_command(SetPixelFormat::new(pf))?;
        di.write_raw(SH8601Commands::SH8601_W_PIXFMT as u8, &[0x05])?;

        let madctl = SetAddressMode::from(options);
        // di.write_command(madctl)?;

        di.write_raw(SH8601Commands::SH8601_C_DISPON as u8, &[])?;

        di.write_raw(SH8601Commands::SH8601_W_WCTRLD1 as u8, &[0x28])?;

        di.write_raw(SH8601Commands::SH8601_W_WDBRIGHTNESSVALNOR as u8, &[0x00])?;

        di.write_raw(SH8601Commands::SH8601_W_WCE as u8, &[0x00])?;

        di.write_raw(SH8601Commands::SH8601_C_INVOFF as u8, &[])?;

        delay.delay_us(100_000);

        di.write_raw(SH8601Commands::SH8601_W_WDBRIGHTNESSVALNOR as u8, &[0x55])?;

        Ok(madctl)
    }
}
