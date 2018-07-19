/// Proove packet structure is
/// ```
/// HHHH HHHH HHHH HHHH HHHH HHHH HHGO CCEE
/// \                               \\ \ \____________ device id
///  \                               \\ \_____________ group id
///   \                               \\______________ device switch
///    \                               \______________ group switch
///     \_____________________________________________ house id
/// ```
///
/// Using only 2 bits for device and group id can be severly limiting. Therefore
/// you can specfiy `enable_compat=false` in the config. Then ids will be spread
/// out on house, group and device ids. This disallows the use of the group
/// switch, thus switching a group will switch each device one by one, but allow
/// for greater flexibility
use failure::Error;

use std::collections::HashMap;

const HOUSE_CODE_OFFSET: u32 = 6;
const HOUSE_CODE_MASK: u32 = 0xFFFF_FFC0;
const GROUP_SWITCH_OFFSET: u32 = 5;
const GROUP_SWITCH_MASK: u32 = 0x20;
const DEVICE_SWITCH_OFFSET: u32 = 4;
const DEVICE_SWITCH_MASK: u32 = 0x10;
const CHANNEL_OFFSET: u32 = 2;
const CHANNEL_MASK: u32 = 0xC;
const UNIT_OFFSET: u32 = 0;
const UNIT_MASK: u32 = 0x3;

#[derive(Debug)]
pub struct Group {
    house_id: Option<u64>,
    group_id: Option<u64>,
    devices: HashMap<String, Device>,
    tries: Option<usize>,
}

impl Group {
    pub fn new(
        house_id: Option<u64>,
        group_id: Option<u64>,
        devices: HashMap<String, Device>,
        tries: Option<usize>,
    ) -> Self {
        Group {
            house_id,
            group_id,
            devices,
            tries,
        }
    }

    pub fn get_proove_packet(&self, status: bool) -> Option<u32> {
        let mut packet = 0;
        packet |= ((self.group_id? as u32) << CHANNEL_OFFSET) & CHANNEL_MASK;
        packet |= ((self.house_id? as u32) << HOUSE_CODE_OFFSET) & HOUSE_CODE_MASK;
        packet |= ((status as u32) << GROUP_SWITCH_OFFSET) & GROUP_SWITCH_MASK;
        Some(packet)
    }
}

#[derive(Debug)]
pub struct Device {
    house_id: u64,
    group_id: u64,
    device_id: u64,
    tries: usize,
}

impl Device {
    pub fn new(house_id: u64, group_id: u64, device_id: u64, tries: usize) -> Self {
        Device {
            house_id,
            group_id,
            device_id,
            tries,
        }
    }

    pub fn get_proove_packet(&self, status: bool) -> u32 {
        let mut packet = 0;
        packet |= ((self.device_id as u32) << UNIT_OFFSET) & UNIT_MASK;
        packet |= ((self.group_id as u32) << CHANNEL_OFFSET) & CHANNEL_MASK;
        packet |= ((self.house_id as u32) << HOUSE_CODE_OFFSET) & HOUSE_CODE_MASK;
        packet |= ((status as u32) << DEVICE_SWITCH_OFFSET) & DEVICE_SWITCH_MASK;
        packet
    }
}

#[derive(Debug, Fail)]
enum ManagementError {
    #[fail(display = "Group '{}' not found.", group_name)]
    GroupNotFound { group_name: String },
    #[fail(display = "Device '{}' not found.", device_name)]
    DeviceNotFound { device_name: String },
    #[fail(display = "Library inconsistency, this should not happen. Please report to authors.")]
    Inconsistency,
}

pub trait PacketSender {
    fn send_packet(&mut self, packet: u32);
}

pub struct DeviceManager<T: PacketSender> {
    groups: HashMap<String, Group>,
    tx: T,
}

impl<T: PacketSender> DeviceManager<T> {
    pub fn new(groups: HashMap<String, Group>, tx: T) -> Self {
        DeviceManager { groups, tx }
    }

    pub fn set_group_state(&mut self, group_name: String, state: bool) -> Result<(), Error> {
        let group = self
            .groups
            .get(&group_name)
            .ok_or(ManagementError::GroupNotFound { group_name })?;
        if let Some(tries) = group.tries {
            let packet = group
                .get_proove_packet(state)
                .ok_or(ManagementError::Inconsistency)?;
            println!("packet: {:032b}", packet);
            for _ in 0..tries {
                self.tx.send_packet(packet);
            }
        } else {
            for device in group.devices.values() {
                let packet = device.get_proove_packet(state);
                println!("packet: {:032b}", packet);
                for _ in 0..device.tries {
                    self.tx.send_packet(packet);
                }
            }
        }
        Ok(())
    }

    pub fn set_device_state(
        &mut self,
        group_name: String,
        device_name: String,
        state: bool,
    ) -> Result<(), Error> {
        let group = self
            .groups
            .get(&group_name)
            .ok_or(ManagementError::GroupNotFound { group_name })?;
        let device = group
            .devices
            .get(&device_name)
            .ok_or(ManagementError::DeviceNotFound { device_name })?;
        let packet = device.get_proove_packet(state);
        println!("packet: {:032b}", packet);
        for _ in 0..device.tries {
            self.tx.send_packet(packet);
        }
        Ok(())
    }
}
