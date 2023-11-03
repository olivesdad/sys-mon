/*
- This mod should handle the logic and state of the app
*/
extern crate systemstat;
use futures::channel::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::sync::mpsc::channel;

use std::time::Duration;
use std::{collections::HashMap, thread};
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

// Poller to check syst monitor
#[derive(Default)]
pub struct Poller {}
impl Poller {
    pub fn new() -> Self {
        Poller::default()
    }

    pub fn sys_mon(&mut self, tx: mpsc::SyncSender<Loads>) {
        let sys = System::new();
        //set cpu temp
        loop {
            let mut loads = Loads::new();

            // set load
            match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    thread::sleep(Duration::from_millis(250));
                    let cpu = cpu.done().unwrap();
                    loads.user = Some(cpu.user * 100.0);
                    loads.nice = Some(cpu.nice * 100.0);
                    loads.system = Some(cpu.system * 100.0);
                    loads.interrupt = Some(cpu.interrupt * 100.0);
                    loads.idle = Some(cpu.idle * 100.0);
                }
                Err(_) => {
                    // set to None if we error
                    loads.user = None;
                    loads.nice = None;
                    loads.system = None;
                    loads.interrupt = None;
                    loads.idle = None;
                }
            }

            //set cpu temp
            match sys.cpu_temp() {
                Ok(cpu_temp) => {
                    loads.temp = Some(cpu_temp);
                }
                Err(_) => {
                    loads.temp = None;
                }
            }

            let res = tx.send(loads);
            if res.is_ok() {
            } else {
                break;
            }
        }
    }
}

pub struct Loads {
    nice: Option<f32>,
    user: Option<f32>,
    system: Option<f32>,
    interrupt: Option<f32>,
    idle: Option<f32>,
    temp: Option<f32>,
}

impl Loads {
    pub fn new() -> Loads {
        Loads {
            nice: None,
            user: None,
            system: None,
            interrupt: None,
            idle: None,
            temp: None,
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
    reciever: Option<mpsc::Receiver<Loads>>,
}

impl App {
    pub fn new() -> App {
        App {
            temp: None,
            load: Loads::new(),
            units: Units::Celcius,
            state: State::run,
            sys: System::new(),
            reciever: None,
        }
    }
    pub fn set_reciever(&mut self, rx: mpsc::Receiver<Loads>) {
        self.reciever = Some(rx);
    }
}

impl App {
    pub fn get_temp(&self) -> f32 {
        match self.units {
            Units::Celcius => self.load.temp.unwrap_or(9999.9999),
            Units::Farenheight => self.load.temp.unwrap_or(9999.9999) * (9.0 / 5.0) + 32.0,
        }
    }

    // Returns an array of loads in f32 in the order of:
    // [ user, nice, system, interrupt, idle]
    pub fn get_load(&self) -> HashMap<String, f32> {
        let mut loads = HashMap::new();

        loads.insert("user".to_string(), self.load.user.unwrap_or(9999.99));
        loads.insert("nice".to_string(), self.load.nice.unwrap_or(9999.99));
        loads.insert("system".to_string(), self.load.system.unwrap_or(9999.99));
        loads.insert(
            "interrupt".to_string(),
            self.load.interrupt.unwrap_or(9999.99),
        );
        loads.insert("idle".to_string(), self.load.idle.unwrap_or(9999.99));

        loads
    }

    pub fn poll(&mut self) {

        // set values
        match &self.reciever {
            //pull load off channel
            Some(rx) => {
                if let Ok(loads) = rx.recv_timeout(Duration::from_millis(300)) {
                    self.load = loads;
                }
            }
            _ => {}
        }
    }
}