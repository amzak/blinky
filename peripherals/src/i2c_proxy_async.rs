use critical_section::Mutex;
use embedded_hal::i2c::{ErrorType, I2c};
use std::cell::RefCell;
use std::sync::Arc;

pub struct I2cProxyAsync<T> {
    bus: Arc<Mutex<RefCell<T>>>,
}

impl<T> I2cProxyAsync<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus: Arc::new(Mutex::new(RefCell::new(bus))),
        }
    }
}

impl<T> Clone for I2cProxyAsync<T> {
    fn clone(&self) -> Self {
        Self {
            bus: self.bus.clone(),
        }
    }
}

impl<T> ErrorType for I2cProxyAsync<T>
where
    T: I2c,
{
    type Error = T::Error;
}

impl<T> I2c for I2cProxyAsync<T>
where
    T: I2c,
{
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        critical_section::with(|cs| {
            let bus = &mut *self.bus.borrow_ref_mut(cs);
            bus.read(address, read)
        })
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        critical_section::with(|cs| {
            let bus = &mut *self.bus.borrow_ref_mut(cs);
            bus.write(address, write)
        })
    }

    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        critical_section::with(|cs| {
            let bus = &mut *self.bus.borrow_ref_mut(cs);
            bus.write_read(address, write, read)
        })
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        critical_section::with(|cs| {
            let bus = &mut *self.bus.borrow_ref_mut(cs);
            bus.transaction(address, operations)
        })
    }
}
