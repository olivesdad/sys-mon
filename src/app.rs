/*
- This mod should handle the logic and state of the app
*/
extern crate systemstat;

use std::{thread, collections::HashMap};
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

struct Loads {
    nice: Option<f32>,
    user: Option<f32>,
    system: Option<f32>,
    interrupt: Option<f32>,
    idle: Option<f32>,
}

impl Loads {
    pub fn new() -> Loads {
        Loads {
            nice: None,
            user: None,
            system: None,
            interrupt: None,
            idle: None,
        }
    }
}

//this struct should handlle the state of the app
pub struct App {
    temp: Option<f32>,
    load: Loads,
    pub units: Units,
    state: State,
    sys: System,
}

impl App {
    pub fn new() -> App {
        App {
            temp: None,
            load: Loads::new(),
            units: Units::Celcius,
            state: State::run,
            sys: System::new(),
        }
    }
}

impl App {
    pub fn get_temp(&self) -> f32 {
        match self.units {
            Units::Celcius => self.temp.unwrap_or(9999.9999),
            Units::Farenheight => self.temp.unwrap_or(9999.9999) * (9.0 / 5.0) + 32.0,
        }
    }

    // Returns an array of loads in f32 in the order of: 
    // [ user, nice, system, interrupt, idle]
    pub fn get_load(&self) -> HashMap<String, f32>{
        let mut loads = HashMap::new();
        
        loads.insert("user".to_string(), self.load.user.unwrap_or(9999.99));
        loads.insert("nice".to_string(),  self.load.nice.unwrap_or(9999.99));
        loads.insert( "system".to_string(), self.load.system.unwrap_or(9999.99));
        loads.insert("interrupt".to_string(), self.load.interrupt.unwrap_or(9999.99));
        loads.insert("idle".to_string(),  self.load.idle.unwrap_or(9999.99));
        
        loads
    }

    pub fn poll(&mut self) {
        //set cpu temp
        match self.sys.cpu_temp() {
            Ok(cpu_temp) => {
                self.temp = Some(cpu_temp);
            }
            Err(_) => {
                self.temp = None;
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
                self.load.user = Some(cpu.user * 100.0);
                self.load.nice = Some(cpu.nice * 100.0);
                self.load.system = Some(cpu.system * 100.0);
                self.load.interrupt = Some(cpu.interrupt * 100.0);
                self.load.idle = Some(cpu.idle * 100.0);
            }
            Err(x) => {
                self.load.user = None;
                self.load.nice = None;
                self.load.system = None;
                self.load.interrupt = None;
                self.load.idle = None;
            }
        }
    }
}
