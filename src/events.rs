/*
    listener for  key presses
*/
use crossterm;
use crossterm::event::KeyCode;
use std::sync::mpsc;

pub enum KeyActions {
    quit,
    toggle_units,
}
pub struct KeyPressHandler {
    tick_rate: std::time::Duration,
    sender: mpsc::Sender<Option<KeyActions>>,
}

impl KeyPressHandler {
    pub fn new(sender: mpsc::Sender<Option<KeyActions>>) -> Self {
        KeyPressHandler {
            sender,
            tick_rate: std::time::Duration::from_millis(250),
        }
    }

    pub fn poll(&mut self) {
        let mut channel_status = Ok(());

        // loop until the channel tries to send into poop
        while channel_status.is_ok() {
            // Poll until timeout so we dont hang forever
            if crossterm::event::poll(self.tick_rate).is_ok() {
                // if the read gets something it should be an OK so match it
                if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                    // I guess this is for if its a keypress
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        // We only care about tab and q
                        match key.code {
                            KeyCode::Tab => channel_status = self.sender.send(Some(KeyActions::toggle_units)),
                            KeyCode::Char('q') => channel_status = self.sender.send(Some(KeyActions::quit)),
                            //Just send None if its a key we dont care about
                            _ => channel_status = self.sender.send(None),
                        }
                    }
                } else {
                    // Just send nothing so we can not hang the listener thread
                    channel_status = self.sender.send(None);
                }
            }
        }
    }
}
