use app::App;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::sync::mpsc::sync_channel;
use std::{error::Error, io, thread, time};

mod UI;
mod app;
mod events;
mod systemstat_example;

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

    let _res: Result<bool, io::Error> = run_app(&mut terminal, &mut app);

    // clean up
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    // create channels
    let (tx, rx) = sync_channel(2);
    // spawn worker for system stats
    let mut poller = app::Poller::new();
    let _worker = thread::spawn(move || poller.sys_mon(tx));
    //set reciever for system stats
    app.set_reciever(rx);

    // spawn event KeyPressHandler
    let (tx, rx) = std::sync::mpsc::channel::<Option<events::KeyActions>>();
    let mut kph = events::KeyPressHandler::new(tx);
    app.set_event_handleer(rx);
    let _event_handler = thread::spawn(move || kph.poll());

    // Draw loop
    loop {
        match app.state {
            app::State::Quit => break,
            _ => {}
        }
        app.poll();
        if app.check_keys().is_err() {
            break;
        }
        //render terminal
        terminal.draw(|f| UI::ui(f, app))?;
        thread::sleep(time::Duration::from_millis(300));
    }

    // if worker.join().is_ok(){}
    Ok(true)
}
