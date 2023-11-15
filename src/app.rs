/*
- This mod should handle the logic and state of the app
*/
extern crate systemstat;
use crate::events::KeyActions;
use bytesize::ByteSize;
use std::sync::mpsc;
use std::time::Duration;
use std::{collections::HashMap, thread};
use systemstat::{saturating_sub_bytes, Platform, System};

pub enum Units {
    Celcius,
    Fahrenheit,
}

pub enum GraphType {
    SparkLine,
    Scatter,
}
pub enum State {
    Run,
    Quit,
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
                    let h = (battery.remaining_time.as_secs() / 3600) as u32;
                    let m = (battery.remaining_time.as_secs() % 60) as u32;
                    loads.battery = Some((battery.remaining_capacity * 100.0) as u8);
                    loads.battery_time = Some((h, m));

                    // set the battery color
                    if let Some(x) = &loads.battery {
                        if *x >= 65 {
                            loads.battery_color = ratatui::style::Color::LightGreen;
                        } else if *x < 65 && *x >= 25 {
                            loads.battery_color = ratatui::style::Color::LightYellow;
                        } else {
                            loads.battery_color = ratatui::style::Color::LightRed;
                        }
                    } else {
                        loads.battery_color = ratatui::style::Color::Red;
                    }
                }
                Err(_) => loads.battery = None,
            }

            // set memory usage
            match sys.memory() {
                Ok(mem) => {
                    loads.mem = Some((saturating_sub_bytes(mem.total, mem.free), mem.total));
                }
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
    battery_time: Option<(u32, u32)>,
    battery_color: ratatui::style::Color,
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
            battery_time: None,
            battery_color: ratatui::style::Color::Red,
        }
    }
}

//this struct should handlle the state of the app
pub struct App {
    pub load: Loads,
    pub units: Units,
    pub state: State,
    pub graph: GraphType,
    temp_vec: Vec<(f64, f64)>,
    temp_vec_f: Vec<(f64, f64)>,
    reciever: Option<mpsc::Receiver<Loads>>,
    event_handler: Option<mpsc::Receiver<Option<KeyActions>>>,
}

impl App {
    pub fn new() -> App {
        App {
            load: Loads::new(),
            units: Units::Celcius,
            state: State::Run,
            reciever: None,
            graph: GraphType::Scatter,
            temp_vec: Vec::new(),
            temp_vec_f: Vec::new(),
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
            Units::Fahrenheit => self.load.temp.unwrap_or(9999.9999) * (9.0 / 5.0) + 32.0,
        }
    }

    // returns a slice of our vector of temp points....
    pub fn get_temp_points<'a>(&'a self) -> &'a[(f64, f64)] {
        match self.units {
           Units::Celcius => {
            let slice = &self.temp_vec[0..(self.temp_vec.len())];
            slice.try_into().unwrap()
           },
           Units::Fahrenheit => {
            let slice = &self.temp_vec_f[0..(self.temp_vec.len())];
            slice.try_into().unwrap()
            },
           }
        }
    

    pub fn get_battery_color(&self) -> ratatui::style::Color {
        self.load.battery_color
    }
    // Get memory return tuple of used, total maybe string is fine
    pub fn get_mem(&self) -> (String, String) {
        match self.load.mem {
            Some(mem) => (mem.0.to_string(), mem.1.to_string()),
            None => ("NA".to_owned(), "NA".to_owned()),
        }
    }
    // gets battery as u8
    pub fn get_battery_left(&self) -> u8 {
        if let Some(p) = self.load.battery {
            p
        } else {
            0
        }
    }

    //Get battery time left
    pub fn get_battery_time(&self) -> String {
        if let Some((h, m)) = self.load.battery_time {
            format!("Time Remaining: {}h {}m", h, m)
        } else {
            "Err".to_owned()
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
                    // Temp Vector for chart. Going to limit data points to 10k so we dont just eat memory
                    if self.temp_vec.len() > 10000 {
                        self.temp_vec = Vec::new();
                    }
                    // push new value on
                    self.temp_vec
                        .push((self.temp_vec.len() as f64, loads.temp.unwrap_or(0.0) as f64));
                    self.temp_vec_f
                        .push((self.temp_vec_f.len() as f64, ((loads.temp.unwrap_or(0.0)*9.0/5.0) + 32.0) as f64));
                    // Replace Loads struct
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
                                KeyActions::Quit => {
                                    self.state = State::Quit;
                                    got_message = false;
                                }
                                KeyActions::ToggleUnits => match self.units {
                                    Units::Celcius => self.units = Units::Fahrenheit,
                                    Units::Fahrenheit => self.units = Units::Celcius,
                                },
                                KeyActions::ClearTemp => {
                                    self.temp_vec = Vec::new();
                                }
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
