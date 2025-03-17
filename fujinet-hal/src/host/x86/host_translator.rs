use async_trait::async_trait;
use std::borrow::Cow;
use crate::device::DeviceResult;
use crate::host::HostTranslator;

/// X86 host translator implementation
/// Since we're targeting x86 for both host and platform, this is a simple pass-through implementation
pub struct X86HostTranslator;

impl X86HostTranslator {
    /// Creates a new X86 host translator
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HostTranslator for X86HostTranslator {
    async fn initialize(&mut self) -> DeviceResult<()> {
        Ok(())
    }

    async fn process_host_data<'a>(&'a mut self, data: &'a [u8]) -> DeviceResult<Cow<'a, [u8]>> {
        // For x86, we can pass through the data without transformation
        Ok(Cow::Borrowed(data))
    }

    async fn process_device_data<'a>(&'a mut self, data: &'a [u8]) -> DeviceResult<Cow<'a, [u8]>> {
        // For x86, we can pass through the data without transformation
        Ok(Cow::Borrowed(data))
    }
}
