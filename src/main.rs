use std::{error::Error, io, thread, time};
use app::App;
use ratatui::{
    backend::{Backend, CrosstermBackend, self},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod app;
mod UI;

fn main() -> Result<(), Box<dyn Error>> {

    //setup terminal
    enable_raw_mode()?;
    //use to log to stderr
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;

    // elements
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    //create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app); 

    // clean up
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| UI::ui(f, app))?;

        //temp  to exit without hanging
        thread::sleep(time::Duration::from_millis(5000));
        break;
    }
    Ok(true)
}