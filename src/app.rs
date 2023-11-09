/*
- This mod should handle the logic and state of the app
*/
extern crate systemstat;
use crate::events::KeyActions;
use bytesize::ByteSize;
use crossterm::event::KeyCode;
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
    quit,
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

            // set battery temp
            match sys.battery_life() {
                Ok(battery) => {
                 //   "\nBattery: {}%, {}h{}m remaining",
                 //   battery.remaining_capacity * 100.0,
                  //  battery.remaining_time.as_secs() / 3600,
                  //  battery.remaining_time.as_secs() % 60
                  loads.battery = Some((battery.remaining_capacity * 100.0) as u8 );
                },
                Err(_) => loads.battery = None,
            }

            // set memory usage
            match sys.memory() {
                Ok(mem) => {
/*                     "\nMemory: {} used / {} ({} bytes) total ({:?})",
                    saturating_sub_bytes(mem.total, mem.free),
                    mem.total,
                    mem.total.as_u64(),
                    mem.platform_memory; */
                    loads.mem = Some((saturating_sub_bytes(mem.total, mem.free), mem.total));
                },
                Err(_) => loads.mem = None,
            }

            // Send results
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
    battery: Option<u8>,
    mem: Option<(ByteSize, ByteSize)>,
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
            battery: None,
            mem: None,
        }
    }
}

//this struct should handlle the state of the app
pub struct App {
    load: Loads,
    pub units: Units,
    pub state: State,
    reciever: Option<mpsc::Receiver<Loads>>,
    event_handler: Option<mpsc::Receiver<Option<KeyActions>>>,
}

impl App {
    pub fn new() -> App {
        App {
            load: Loads::new(),
            units: Units::Celcius,
            state: State::run,
            reciever: None,
            event_handler: None,
        }
    }
    pub fn set_event_handleer(&mut self, rx: mpsc::Receiver<Option<KeyActions>>) {
        self.event_handler = Some(rx);
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
    // Get memory return tuple of used, total maybe string is fine
    pub fn get_mem(&self) -> (String, String){
        match self.load.mem {
            Some(mem) =>{
                (mem.0.to_string(), mem.1.to_string())
            },
            None =>{
                ("NA".to_owned(), "NA".to_owned())
            }
        }
    }
    // gets battery as u8
    pub fn get_battery_left(&self) ->u8 {
        if let Some(p) = self.load.battery {
            p
        } else {
            0
        }
    }
    // get hashmap for temp things
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
                if let Ok(loads) = rx.recv_timeout(Duration::from_millis(250)) {
                    self.load = loads;
                }
            }
            _ => {}
        }
    }

    // Function to check if key presses were entered
    pub fn check_keys(&mut self) -> Result<(), ()> {
        let mut got_message = true;

        match &self.event_handler {
            None => return Err(()),
            Some(x) => {
                while got_message {
                    let res = x.recv_timeout(Duration::from_millis(250));
                    match res {
                        Err(_) => got_message = false,
                        Ok(msg) => match msg {
                            Some(key) => match key {
                                KeyActions::quit => {
                                    self.state = State::quit;
                                    got_message = false;
                                }
                                KeyActions::toggle_units => match self.units {
                                    Units::Celcius => self.units = Units::Farenheight,
                                    Units::Farenheight => self.units = Units::Celcius,
                                },
                            },
                            None => {}
                        },
                    }
                }
            }
        }
        Ok(())
    }
}
