use devices::{Device, Group};
use failure::Error;
use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use toml;

const DEFAULT_PATH: &str = "~/.proove_mqtt.toml";

const MAX_GID: u64 = 4;
const MAX_DID: u64 = 4;
const MAX_HID: u64 = 67_108_864; // 2^26
const MAX_ID: u64 = 1_073_741_824; // 2^30

fn derive_ids(mut id: u64) -> (u64, u64, u64) {
    let x = id & 0b11;
    id >>= 2;
    let y = id & 0b11;
    id >>= 2;
    let z = id;
    (z, y, x)
}

#[derive(Debug, Fail)]
enum ConfigError {
    #[fail(display = "If compat enabled, groups must have an id.")]
    GIDNotSet,
    #[fail(display = "If compat disabled, groups are not allowed to have an id.")]
    GIDSet,
    #[fail(display = "If compat enabled, groups must have a house id.")]
    HIDNotSet,
    #[fail(display = "If compat disabled, groups are not allowed to have a house id.")]
    HIDSet,
    #[fail(display = "If compat disabled, groups must have a unique name.")]
    NameNotSet,
    #[fail(display = "Group name {} not unique.", group_name)]
    GroupNotUnique { group_name: String },
    #[fail(display = "Device name {} not unique.", device_name)]
    DeviceNotUnique { device_name: String },
    #[fail(
        display = "{} is too large for a {}; maximum is {}. Disable compat for larger device ids or use multiple groups",
        value,
        name,
        max
    )]
    OutOfBounds { value: u64, name: String, max: u64 },
}

fn default_topic() -> String {
    "proove".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub tx_pin: u64,
    enable_compat: bool,
    broker: String,
    #[serde(default = "default_topic")]
    root_topic: String,
    tries: usize,
    house_id: Option<u64>,
    groups: Vec<GroupConfig>,
}

#[derive(Debug, Deserialize)]
struct GroupConfig {
    enable_compat: Option<bool>,
    group_id: Option<u64>,
    house_id: Option<u64>,
    name: Option<String>,
    tries: Option<usize>,
    devices: Vec<DeviceConfig>,
}

#[derive(Debug, Deserialize)]
struct DeviceConfig {
    device_id: u64,
    name: Option<String>,
    tries: Option<usize>,
}

pub fn load_config(path: Option<&str>) -> Result<Config, Box<error::Error>> {
    let config_path = path.unwrap_or(DEFAULT_PATH);

    let mut string = String::new();
    File::open(config_path)?.read_to_string(&mut string)?;

    let config = toml::from_str(&string)?;
    Ok(config)
}

// TODO clones are ugly
pub fn create_devices(config: Config) -> Result<HashMap<String, Group>, Error> {
    let mut groups = HashMap::with_capacity(config.groups.len());
    for gc in config.groups {
        let mut devices = HashMap::with_capacity(gc.devices.len());
        let group = if gc.enable_compat.unwrap_or(config.enable_compat) {
            let group_id = gc.group_id.ok_or(ConfigError::GIDNotSet)?;
            let house_id = gc
                .house_id
                .or(config.house_id)
                .ok_or(ConfigError::HIDNotSet)?;

            ensure!(
                group_id < MAX_GID,
                ConfigError::OutOfBounds {
                    value: group_id,
                    name: "group id".to_owned(),
                    max: MAX_GID
                }
            );
            ensure!(
                house_id < MAX_HID,
                ConfigError::OutOfBounds {
                    value: house_id,
                    name: "house id".to_owned(),
                    max: MAX_HID
                }
            );

            let tries = gc.tries.unwrap_or(config.tries);
            let group_name = gc.name.unwrap_or(format!("{}.{}", group_id, house_id));

            for dc in gc.devices {
                let device_name = dc.name.unwrap_or(format!("{}", dc.device_id));
                let tries = dc.tries.unwrap_or(tries);

                ensure!(
                    dc.device_id < MAX_DID,
                    ConfigError::OutOfBounds {
                        value: dc.device_id,
                        name: "device id".to_owned(),
                        max: MAX_DID
                    }
                );
                ensure!(
                    devices
                        .insert(
                            device_name.clone(),
                            Device::new(house_id, group_id, dc.device_id, tries)
                        )
                        .is_none(),
                    ConfigError::DeviceNotUnique {
                        device_name
                    }
                );
            }
            (
                group_name,
                Group::new(Some(house_id), Some(group_id), devices, Some(tries)),
            )
        } else {
            ensure!(gc.group_id.is_none(), ConfigError::GIDSet);
            ensure!(gc.house_id.is_none(), ConfigError::HIDSet);

            let group_name = gc.name.ok_or(ConfigError::NameNotSet)?;
            let tries = gc.tries.unwrap_or(config.tries);

            for dc in gc.devices {
                let device_name = dc.name.unwrap_or(format!("{}", dc.device_id));
                let tries = dc.tries.unwrap_or(tries);

                ensure!(
                    dc.device_id < MAX_ID,
                    ConfigError::OutOfBounds {
                        value: dc.device_id,
                        name: "global device id".to_owned(),
                        max: MAX_ID
                    }
                );

                let ids = derive_ids(dc.device_id);
                ensure!(
                    devices
                        .insert(device_name.clone(), Device::new(ids.0, ids.1, ids.2, tries))
                        .is_none(),
                    ConfigError::DeviceNotUnique { device_name }
                );
            }
            (group_name, Group::new(None, None, devices, None))
        };
        ensure!(
            groups.insert(group.0.clone(), group.1).is_none(),
            ConfigError::GroupNotUnique {
                group_name: group.0
            }
        );
    }
    Ok(groups)
}
