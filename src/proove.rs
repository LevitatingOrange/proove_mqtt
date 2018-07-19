use failure::Error;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

use devices::PacketSender;

const PULSE_HIGH: Duration = Duration::from_micros(250);
const PULSE_ONE_LOW: Duration = Duration::from_micros(250);
const PULSE_ZERO_LOW: Duration = Duration::from_micros(1250);
const PULSE_SYNC_LOW: Duration = Duration::from_micros(2500);
const PULSE_PAUSE_LOW: Duration = Duration::from_micros(10000);

pub struct Proove {
    tx_pin: Pin,
}

impl Proove {
    pub fn new(pin: u64) -> Result<Self, Error> {
        let pin = Pin::new(pin);
        pin.export()?;
        pin.set_direction(Direction::Out)?;
        pin.set_value(0)?;
        Ok(Proove { tx_pin: pin })
    }

    fn send_pulse(&mut self, value: bool) {
        self.tx_pin.set_value(1).unwrap();
        sleep(PULSE_HIGH);
        self.tx_pin.set_value(0).unwrap();
        sleep(if value { PULSE_ONE_LOW } else { PULSE_ZERO_LOW });
    }

    fn send_bit(&mut self, value: u32) {
        if value != 0 {
            self.send_pulse(true);
            self.send_pulse(false);
        } else {
            self.send_pulse(false);
            self.send_pulse(true);
        }
    }

    fn send_sync(&mut self) {
        self.tx_pin.set_value(1).unwrap();
        sleep(PULSE_HIGH);
        self.tx_pin.set_value(0).unwrap();
        sleep(PULSE_SYNC_LOW);
    }

    fn send_pause(&mut self) {
        self.tx_pin.set_value(1).unwrap();
        sleep(PULSE_HIGH);
        self.tx_pin.set_value(0).unwrap();
        sleep(PULSE_PAUSE_LOW);
    }
}

impl PacketSender for Proove {
    fn send_packet(&mut self, packet: u32) {
        self.send_sync();
        for i in (0..32).rev() {
            self.send_bit((packet >> i) & 1);
        }
        self.send_pause();
    }
}
