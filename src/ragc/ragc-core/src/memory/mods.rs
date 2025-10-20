/// Base trait for interrupt-capable peripherals
pub trait Peripheral {
    /// Check interrupt status and return identifier
    fn is_interrupt(&mut self) -> u16;
}

/// Interface for channel-based I/O devices
pub trait IoPeriph {
    /// Read and write from specified channel (port/register)
    fn read(&self, _channel_idx: usize) -> u16;
    fn write(&mut self, channel_idx: usize, value: u16);

    /// Check device-specific interrupt status
    fn is_interrupt(&mut self) -> u16;
}
