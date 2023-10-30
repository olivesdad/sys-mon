/*
- This mod should handle the logic and state of the app
*/

//track which screen is being used
pub enum CurrentScreen {
    CPUTemp,
    CPULoad,
    Exiting,
}
//this struct should handlle the state of the app
pub struct App{
    
}

impl App {
    pub fn new() -> App {
        App {}
    }
}