use std::cell::RefCell;
use std::rc::Rc;
use embedded_hal::i2c::{ErrorType, I2c};

pub struct I2cProxy<T> {
    bus: Rc<RefCell<T>>,
}

impl<T> I2cProxy<T> {
    pub fn new(bus: Rc<RefCell<T>>) -> Self {
        Self { bus }
    }
}

impl<T> ErrorType for I2cProxy<T>
where
    T: I2c,
{
    type Error = T::Error;
}

impl<T> I2c for I2cProxy<T>
    where
        T: I2c,
{
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.read(address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.write(address, write)
    }

    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.write_read(address, write, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.transaction(address, operations)
    }
}