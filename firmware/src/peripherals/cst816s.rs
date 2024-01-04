use embedded_hal as hal;
use embedded_hal::blocking::delay::DelayUs;
use num_enum::{IntoPrimitive, TryFromPrimitive};

const BUFFER_SIZE: usize = 10;
const DEFAULT_I2C_ADDRESS: u8 = 0x15;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE, PinE> {
    Comm(CommE),
    Pin(PinE),
    GenericError
}

pub struct CST816S<I2C, PIN_INT, PIN_RST> {
    i2c: I2C,
    pin_int: PIN_INT,
    pin_rst: PIN_RST,

    buffer: [u8; BUFFER_SIZE]
}

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Action {
    Down = 0x00,
    Lift = 0x01,
    Contact = 0x02
}

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum TouchGesture {
    None = 0x00,
    SlideDown = 0x01,
    SlideUp = 0x02,
    SlideLeft = 0x03,
    SlideRight = 0x04,
    SingleClick = 0x05,
    DoubleClick = 0x0B,
    LongPress = 0x0C,
}

#[repr(u8)]
#[derive(IntoPrimitive)]
pub enum Reg {
    TouchData = 0x00,
    Version = 0x15,
    VersionInfo = 0xA7,
    GestureEn = 0xD0,
    GestureOutputAddress = 0xD3
}

#[derive(Debug)]
pub struct TouchEvent {
    pub x: i32,
    pub y: i32,
    /// the gesture that this touch is part of
    pub gesture: TouchGesture,
    /// 0 down, 1 lift, 2 contact
    pub action: Action,
    /// identifies the finger that touched (0-9)
    pub finger_id: u8,
    /// pressure level of touch
    pub pressure: u8,
    /// the surface area of the touch
    pub area: u8,
}

pub struct DeviceInfo {
    pub Version: u8,
    pub VersionInfo: [u8; 3]
}

impl<I2C, PIN_INT, PIN_RST, CommE, PinE> CST816S<I2C, PIN_INT, PIN_RST>
    where
        I2C: hal::blocking::i2c::Write<Error = CommE>
             + hal::blocking::i2c::Read<Error = CommE>
             + hal::blocking::i2c::WriteRead<Error = CommE>,
        PIN_INT: hal::digital::v2::InputPin,
        PIN_RST: hal::digital::v2::StatefulOutputPin<Error = PinE>,
{
    pub fn new(port: I2C, interrupt_pin: PIN_INT, reset_pin: PIN_RST) -> Self {
        Self {
            i2c: port,
            pin_int: interrupt_pin,
            pin_rst: reset_pin,
            buffer: [0; BUFFER_SIZE]
        }
    }

    pub fn setup(&mut self, delay_source: &mut impl DelayUs<u32>) -> Result<(), Error<CommE, PinE>> {
        // reset the chip
        self.pin_rst.set_low().map_err(Error::Pin)?;
        delay_source.delay_us(20_000);
        self.pin_rst.set_high().map_err(Error::Pin)?;
        delay_source.delay_us(400_000);

        //TODO setup interrupt on pin_int

        self.enable_gestures(true)?;

        Ok(())
    }

    fn read_registers(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.i2c.write_read(
                DEFAULT_I2C_ADDRESS,
                &[Reg::TouchData.into()],
                self.buffer.as_mut(),
            )
            .map_err(Error::Comm)?;
        Ok(())
    }

    pub fn get_device_info(&mut self) -> Result<DeviceInfo, Error<CommE, PinE>> {
        let mut version: [u8; 1] = [0];
        self.i2c.write_read(DEFAULT_I2C_ADDRESS, &[Reg::Version.into()], &mut version).map_err(Error::Comm)?;;
        let mut versionInfo: [u8; 3] = [0; 3];
        versionInfo[0] = Reg::VersionInfo.into();
        self.i2c.write_read(DEFAULT_I2C_ADDRESS, &[Reg::VersionInfo.into()], &mut versionInfo).map_err(Error::Comm)?;

        return Ok(DeviceInfo {
            Version: version[0],
            VersionInfo: versionInfo
        });
    }

    pub fn get_touch_event(&mut self) -> Result<TouchEvent, Error<CommE, PinE>> {
        let mut data: [u8; 10] = [0; 10];
        self.i2c.write_read(DEFAULT_I2C_ADDRESS, &[Reg::TouchData.into()], &mut data).map_err(Error::Comm)?;;

        Ok(TouchEvent {
            gesture: data[1].try_into().unwrap(),
            x: 0, //((data[3] & 0x0F) << 8 + data[4]) as i32,
            y: 0, //((data[5] & 0x0F) << 8 + data[6]) as i32,
            action: (data[3] >> 7).try_into().unwrap(),
            finger_id: 0,
            pressure: 0,
            area: 0,
        })
    }

    pub fn get_data_raw(&mut self, data: &mut [u8; 10]) -> Result<(), Error<CommE, PinE>> {
        self.i2c.write_read(DEFAULT_I2C_ADDRESS, &[Reg::TouchData.into()], data).map_err(Error::Comm)?;
        Ok(())
    }

    pub fn enable_gestures(&mut self, flag: bool) -> Result<(), Error<CommE, PinE>> {
        let mut data: [u8; 2] = [Reg::GestureEn.into(), flag.into()];
        self.i2c.write(DEFAULT_I2C_ADDRESS, &data).map_err(Error::Comm)?;
        Ok(())
    }

    pub fn set_gesture_output_address(&mut self, addr: u8) -> Result<u8, Error<CommE, PinE>> {
        let mut data: [u8; 1] = [addr];
        self.i2c.write_read(DEFAULT_I2C_ADDRESS, &[Reg::GestureOutputAddress.into()], &mut data).map_err(Error::Comm)?;
        Ok(data[0])
    }

}