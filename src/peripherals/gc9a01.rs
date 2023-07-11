use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::IntoStorage;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
use mipidsi::instruction::Instruction;
use mipidsi::models::Model;
use mipidsi::Display;
use mipidsi::{error::InitError, Builder};
use {mipidsi::models::write_command, mipidsi::Error, mipidsi::ModelOptions};

pub struct GC9A01Rgb565;

impl Model for GC9A01Rgb565 {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<u8, InitError<RST::Error>>
        where
            RST: OutputPin,
            DELAY: DelayUs<u32>,
            DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        let madctl = options.madctl();

        write_command_u8(di, 0xEF, &[])?;
        write_command_u8(di, 0xEB, &[0x14])?;
        write_command_u8(di, 0xFE, &[])?;
        write_command_u8(di, 0xEF, &[])?;
        write_command_u8(di, 0xEB, &[0x14])?;
        write_command_u8(di, 0x84, &[0x40])?;
        write_command_u8(di, 0x85, &[0xFF])?;
        write_command_u8(di, 0x86, &[0xFF])?;
        write_command_u8(di, 0x87, &[0xFF])?;
        write_command_u8(di, 0x88, &[0x0A])?;
        write_command_u8(di, 0x89, &[0x21])?;
        write_command_u8(di, 0x8A, &[0x00])?;
        write_command_u8(di, 0x8B, &[0x80])?;
        write_command_u8(di, 0x8C, &[0x01])?;
        write_command_u8(di, 0x8D, &[0x01])?;
        write_command_u8(di, 0x8E, &[0xFF])?;
        write_command_u8(di, 0x8F, &[0xFF])?;
        write_command(di, Instruction::DFC, &[0x00, 0x20])?;

        write_command(di, Instruction::COLMOD, &[0x05])?; // Pixel16Bit

        write_command_u8(di, 0x90, &[0x08, 0x08, 0x08, 0x08])?;
        write_command_u8(di, 0xBD, &[0x06])?;
        write_command_u8(di, 0xBC, &[0x00])?;
        write_command_u8(di, 0xFF, &[0x60, 0x01, 0x04])?;

        write_command(di, Instruction::PWR4, &[0x13])?;
        write_command(di, Instruction::PWR5, &[0x13])?;

        write_command_u8(di, 0xC9, &[0x22])?; // Vreg2aVoltageControl
        write_command_u8(di, 0xBE, &[0x11])?;
        write_command_u8(di, 0xE1, &[0x10, 0x0E])?;
        write_command_u8(di, 0xDF, &[0x21, 0x0C, 0x02])?;
        write_command_u8(di, 0xF0, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A])?;
        write_command_u8(di, 0xF1, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F])?;
        write_command_u8(di, 0xF2, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A])?;
        write_command_u8(di, 0xF3, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F])?;
        write_command_u8(di, 0xED, &[0x1B, 0x0B])?;
        write_command_u8(di, 0xAE, &[0x77])?;
        write_command_u8(di, 0xCD, &[0x63])?;

        write_command_u8(
            di,
            0x70,
            &[0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03],
        )?;

        write_command_u8(di, 0xE8, &[0x34])?;

        write_command_u8(
            di,
            0x62,
            &[
                0x18, 0x0D, 0x71, 0xED, 0x70, 0x70, 0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70,
            ],
        )?;
        write_command_u8(
            di,
            0x63,
            &[
                0x18, 0x11, 0x71, 0xF1, 0x70, 0x70, 0x18, 0x13, 0x71, 0xF3, 0x70, 0x70,
            ],
        )?;

        write_command_u8(di, 0x64, &[0x28, 0x29, 0xF1, 0x01, 0xF1, 0x00, 0x07])?;
        write_command_u8(
            di,
            0x66,
            &[0x3C, 0x00, 0xCD, 0x67, 0x45, 0x45, 0x10, 0x00, 0x00, 0x00],
        )?;

        write_command_u8(
            di,
            0x67,
            &[0x00, 0x3C, 0x00, 0x00, 0x00, 0x01, 0x54, 0x10, 0x32, 0x98],
        )?;

        write_command_u8(di, 0x74, &[0x10, 0x85, 0x80, 0x00, 0x00, 0x4E, 0x00])?;
        write_command_u8(di, 0x98, &[0x3E, 0x07])?;
        write_command_u8(di, 0x35, &[])?;

        write_command(di, Instruction::INVON, &[])?;
        write_command(di, Instruction::SLPOUT, &[])?;

        write_command(di, Instruction::MADCTL, &[0x48])?; // display orientation

        delay.delay_us(120_000);

        write_command(di, Instruction::DISPON, &[])?;

        delay.delay_us(30_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
        where
            DI: WriteOnlyDataCommand,
            I: IntoIterator<Item=Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        di.send_data(buf)
    }

    fn default_options() -> ModelOptions {
        let opt = ModelOptions::with_sizes((240, 240), (240, 240));
        return opt;
    }
}

pub struct Builder_GC9A01Rgb565;

impl Builder_GC9A01Rgb565 {
    pub fn create<DI: WriteOnlyDataCommand>(di: DI) -> Builder<DI, GC9A01Rgb565> {
        Builder::with_model(di, GC9A01Rgb565)
    }
}

fn write_command_u8<DI>(di: &mut DI, command: u8, params: &[u8]) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
{
    di.send_commands(DataFormat::U8(&[command]))?;

    if !params.is_empty() {
        di.send_data(DataFormat::U8(params))?;
        Ok(())
    } else {
        Ok(())
    }
}