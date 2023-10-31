/*
- This mod should handle the logic and state of the app
*/
extern crate systemstat;

use std::thread;
use std::time::Duration;
use systemstat::{saturating_sub_bytes, Platform, System};

//track which screen is being used
pub enum Units {
    Celcius,
    Farenheight,
}

pub enum State {
    run,
    exit,
}

//this struct should handlle the state of the app
pub struct App {
    temp: Option<f32>,
    load: Option<f32>,
    pub units: Units,
    state: State,
    sys: System,
}

impl App {
    pub fn new() -> App {
        App {
            temp: None,
            load: None,
            units: Units::Celcius,
            state: State::run,
            sys: System::new(),
        }
    }
}

impl App {
    pub fn getTemp(&self) -> f32 {
        match self.units {
            Units::Celcius => self.temp.unwrap_or(9999.9999),
            Units::Farenheight => self.temp.unwrap_or(9999.9999) * (9.0 / 5.0) + 32.0,
        }
    }

    pub fn getLoad(&self) -> f32 {
        self.load.unwrap_or(9999.9999)
    }

    pub fn poll(&mut self) {
        //set cpu temp
        match self.sys.cpu_temp() {
            Ok(cpu_temp) => {
                self.temp = Some(cpu_temp);
            }
            Err(_) => {
                self.load = None;
            }
        }

        // set load
        match self.sys.cpu_load_aggregate() {
            Ok(cpu) => {
                thread::sleep(Duration::from_millis(250));
                let cpu = cpu.done().unwrap();
                /*                 println!(
                    "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                    cpu.user * 100.0,
                    cpu.nice * 100.0,
                    cpu.system * 100.0,
                    cpu.interrupt * 100.0,
                    cpu.idle * 100.0
                ); */
                self.load = Some(100.0 - (cpu.idle * 100.0));
            }
            Err(x) => {
                self.load = None;
            }
        }
    }
}
