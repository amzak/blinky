use esp_idf_hal::spi::{config::LineWidth, Operation, SpiSingleDeviceDriver};
use esp_idf_sys::EspError;
use mipidsi::interface::{Interface, SpiError};

pub struct SpiInterfaceNoDC<'a> {
    spi: SpiSingleDeviceDriver<'a>,
    buffer: &'a mut [u8],
}

#[derive(Debug)]
pub enum Nothing {}

impl<'a> SpiInterfaceNoDC<'a> {
    pub fn new(spi: SpiSingleDeviceDriver<'a>, buffer: &'a mut [u8]) -> Self {
        Self { spi, buffer }
    }
}

impl<'a> Interface for SpiInterfaceNoDC<'a> {
    type Word = u8;

    type Error = SpiError<EspError, Nothing>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        if command == 0x2c || command == 0x3c {
            log::debug!("cmd: {:02X}, {:02X?}", command, args);
            return Ok(());
        }

        if command == 0x2c || command == 0x3c {
            self.spi
                .transaction(&mut [Operation::WriteWithWidth(
                    &[0x32, 0x00, command, 0x00],
                    LineWidth::Single,
                )])
                .map_err(SpiError::Spi)?;

            return Ok(());
        }

        if args.len() > 0 {
            self.spi
                .transaction(&mut [
                    Operation::WriteWithWidth(&[0x02, 0x00, command, 0x00], LineWidth::Single),
                    Operation::WriteWithWidth(args, LineWidth::Single),
                ])
                .map_err(SpiError::Spi)?;
        } else {
            self.spi
                .transaction(&mut [Operation::WriteWithWidth(
                    &[0x02, 0x00, command, 0x00],
                    LineWidth::Single,
                )])
                .map_err(SpiError::Spi)?;
        }

        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        let mut arrays = pixels.into_iter();

        assert!(self.buffer.len() >= N);

        let mut done = false;

        let mut cmd = 0x2c;

        while !done {
            let mut i = 0;
            for chunk in self.buffer.chunks_exact_mut(N) {
                if let Some(array) = arrays.next() {
                    let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                    *chunk = array;
                    i += N;
                } else {
                    done = true;
                    break;
                };
            }

            self.spi
                .transaction(&mut [
                    Operation::WriteWithWidth(&[0x32, 0x00, cmd, 0x00], LineWidth::Single),
                    Operation::WriteWithWidth(&self.buffer[..i], LineWidth::Quad),
                ])
                .map_err(SpiError::Spi)?;

            cmd = 0x3c;
        }
        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        let fill_count = core::cmp::min(count, (self.buffer.len() / N) as u32);
        let filled_len = fill_count as usize * N;
        for chunk in self.buffer[..(filled_len)].chunks_exact_mut(N) {
            let chunk: &mut [u8; N] = chunk.try_into().unwrap();
            *chunk = pixel;
        }

        let mut index = count;

        let mut command = 0x2c;

        while index >= fill_count {
            self.spi
                .transaction(&mut [
                    Operation::WriteWithWidth(&[0x32, 0x00, command, 0x00], LineWidth::Single),
                    Operation::WriteWithWidth(&self.buffer[..filled_len], LineWidth::Quad),
                ])
                .map_err(SpiError::Spi)?;

            index -= fill_count;
            command = 0x3c;
        }
        if index != 0 {
            self.spi
                .transaction(&mut [
                    Operation::WriteWithWidth(&[0x32, 0x00, 0x3c, 0x00], LineWidth::Single),
                    Operation::WriteWithWidth(
                        &self.buffer[..(index as usize * pixel.len())],
                        LineWidth::Quad,
                    ),
                ])
                .map_err(SpiError::Spi)?;
        }
        Ok(())
    }
}
