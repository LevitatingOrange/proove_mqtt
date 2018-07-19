extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate sysfs_gpio;

mod config;
mod devices;
mod proove;

use std::env::args;
use config::{load_config, create_devices};
use devices::DeviceManager;
use proove::Proove;

fn main() {
    let args: Vec<String> = args().take(2).collect();
    let config = load_config(Some(&args[1])).unwrap();
    let tx = Proove::new(config.tx_pin).unwrap();
    let devices = create_devices(config).unwrap();
    
    let mut manager = DeviceManager::new(devices, tx);

    manager.set_device_state("Lennarts Zimmer".to_owned(), "Klavierlampe".to_owned(), true).unwrap();
    manager.set_group_state("Lennarts Zimmer".to_owned(), true).unwrap();
    manager.set_group_state("Fabis Zimmer".to_owned(), true).unwrap();
    manager.set_device_state("Fabis Zimmer".to_owned(), "Stehlampe".to_owned(), true).unwrap();
}
