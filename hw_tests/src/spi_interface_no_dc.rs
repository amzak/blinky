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
    /// Create new interface
    pub fn new(spi: SpiSingleDeviceDriver<'a>, buffer: &'a mut [u8]) -> Self {
        Self { spi, buffer }
    }
}

impl<'a> Interface for SpiInterfaceNoDC<'a> {
    type Word = u8;

    type Error = SpiError<EspError, Nothing>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        if command == 0x2a || command == 0x2b || command == 0x2c || command == 0x3c {
            //return Ok(());
        }

        if command == 0x2c || command == 0x3c {
            // log::info!("ignored cmd: {:02X}, {:02X?}", command, args);
            return Ok(());
        }

        if args.len() > 0 {
            self.spi
                .transaction(&mut [
                    Operation::WriteWithWidth(&[0x02, 0x00, command, 0x00], LineWidth::Single),
                    Operation::WriteWithWidth(args, LineWidth::Single),
                ])
                .map_err(SpiError::Spi)?;
            // log::info!("cmd: {:02X}, {:02X?}", command, args);
        } else {
            self.spi
                .transaction(&mut [Operation::WriteWithWidth(
                    &[0x02, 0x00, command, 0x00],
                    LineWidth::Single,
                )])
                .map_err(SpiError::Spi)?;

            // log::info!("cmd: {:02X}", command);
        }

        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        // log::info!("send_pixels");

        // let mut buffer = [0u8; 4];

        // const width: usize = 480;
        // const height: usize = 480;

        // let mut data = vec![0x3c; width * height * 2];

        // let start_column: u16 = 0;
        // let end_column: u16 = 100;

        // let region_size: usize = (end_column as usize - start_column as usize + 1)
        //     * (end_column as usize - start_column as usize + 1)
        //     * 2;

        // buffer[0..2].copy_from_slice(&start_column.to_be_bytes());
        // buffer[2..4].copy_from_slice(&end_column.to_be_bytes());

        // self.spi
        //     .transaction(&mut [
        //         Operation::WriteWithWidth(&[0x02, 0x00, 0x2a, 0x00], LineWidth::Single),
        //         Operation::WriteWithWidth(&buffer, LineWidth::Single),
        //     ])
        //     .unwrap();

        // self.spi
        //     .transaction(&mut [
        //         Operation::WriteWithWidth(&[0x02, 0x00, 0x2b, 0x00], LineWidth::Single),
        //         Operation::WriteWithWidth(&buffer, LineWidth::Single),
        //     ])
        //     .unwrap();

        // let range = 0..region_size;

        // self.spi
        //     .transaction(&mut [
        //         Operation::WriteWithWidth(&[0x32, 0x00, 0x2c, 0x00], LineWidth::Single),
        //         Operation::WriteWithWidth(&data[range], LineWidth::Quad),
        //     ])
        //     .unwrap();

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
            // log::info!("pixels: {} {}", i, self.buffer.len());

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
        log::info!("send_repeated_pixel, count: {}", count);

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
