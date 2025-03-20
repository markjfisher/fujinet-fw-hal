use crate::device::network::NetworkUrl;

pub const MAX_NETWORK_DEVICES: usize = 8;

#[derive(Default)]
pub struct DeviceState {
    pub mode: u8,
    pub trans: u8,
    pub url: Option<NetworkUrl>,
}

pub struct DeviceManager {
    devices: [DeviceState; MAX_NETWORK_DEVICES],
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: std::array::from_fn(|_| DeviceState::default()),
        }
    }

    pub fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
        if device_id < MAX_NETWORK_DEVICES {
            Some(&mut self.devices[device_id])
        } else {
            None
        }
    }

    pub fn set_device_state(&mut self, device_id: usize, mode: u8, trans: u8, url: NetworkUrl) -> bool {
        if let Some(device) = self.get_device(device_id) {
            device.mode = mode;
            device.trans = trans;
            device.url = Some(url);
            true
        } else {
            false
        }
    }

    pub fn clear_device_state(&mut self, device_id: usize) -> bool {
        if let Some(device) = self.get_device(device_id) {
            device.mode = 0;
            device.trans = 0;
            device.url = None;
            true
        } else {
            false
        }
    }
}