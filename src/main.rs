extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate sysfs_gpio;

mod config;
mod devices;
mod proove;

use config::{create_devices, load_config};
use devices::DeviceManager;
use proove::Proove;
use std::env::args;

fn main() {
    let args: Vec<String> = args().take(4).collect();
    let config = load_config(Some(&args[1])).unwrap();
    let tx = Proove::new(config.tx_pin).unwrap();
    let devices = create_devices(config).unwrap();

    let mut manager = DeviceManager::new(devices, tx);

    manager
        .set_device_state(
            args[2].clone(),
            args[3].clone(),
            true,
        )
        .unwrap();
}
