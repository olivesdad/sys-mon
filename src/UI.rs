/*
- This module should handle the rendering and layout of the thing
*/

use crate::app::{App, Units};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Gauge},
    Frame,
};

// Main function used to render the UI.
// passed as a closure to the draw function which passes the frame size to it.
// We also pass an app reference to it so we can query the state of hte world
pub fn ui(f: &mut Frame, app: &App) {
    // Sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title Box
    // Border box thing
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title("OOga Booga");
    // Paragraph widget takes ownership of title_block
    let title = Paragraph::new(Text::styled(
        "system stats",
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .block(title_block)
    .alignment(Alignment::Center);

    // SPlit in 2 blocks for info
    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // SPLIT chunk 0 for memory an load
    let loads_mem = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Max(3)])
        .split(info_chunks[0]);

    //SPlit again for temp and battery life
    let battery_temp_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Max(7)])
        .split(info_chunks[1]);

    // +++++++ Battery Block ++++++++++ //
    let battery_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title("Battery Percent");
    // Split again
    let battery_space = battery_block.inner(battery_temp_chunks[1]);
    let battery_recs = Layout::default()
        .constraints([Constraint::Max(3),Constraint::Min(4)])
        .split(battery_space);
    
    // Battery widget Paragraph
    let battery_percent = Paragraph::new(
        Text::styled(app.get_battery_time(), Style::default()))
            .alignment(Alignment::Center);

    // Battery Gauge Widget
    let battery_gauge = Gauge::default()
        .gauge_style(Style::default()
            .fg(Color::Green)
            .bg(Color::DarkGray))
        .percent(app.get_battery_left() as u16);   


    // +++++++ CPUT TEMP BLOCK + PARAGRAPH ++++++++ //
    let temp_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title("CPU Temperature");
    // unit char
    let unit = match app.units {
        Units::Celcius => "C",
        Units::Farenheight => "F",
    };

    let temp = Paragraph::new(Text::styled(
        app.get_temp().to_string() + unit,
        Style::default(),
    ))
    .block(temp_block)
    .alignment(Alignment::Center);


    // +++++++ CPU LOAD BLOCK + PARAGRAPH  ++++++++ //
    let load_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title("System Load");

    // Lines for loads
    let loads = app.get_load();
    let load_lines = vec![
        format!("nice: {}%", loads.get("nice").unwrap()).into(),
        format!("user: {}%", loads.get("user").unwrap()).into(),
        format!("system {}%", loads.get("system").unwrap()).into(),
        format!("interrupt: {}%", loads.get("interrupt").unwrap()).into(),
        format!("idle: {}%", loads.get("idle").unwrap()).into(),
    ];

    let load = Paragraph::new(load_lines)
        .block(load_block)
        .alignment(Alignment::Left);

    // ++++++++ MEMORY USAGE BLOCK -++++++++//
    let mem_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title("Memory Usage");

    let (x, y )= app.get_mem();
    let memory = Paragraph::new(
        Text::styled(
            format!("{} Used / {} Total", x,y),Style::default()))
            .block(mem_block);
    

    //Quit message box
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::LightYellow));
    // Quit paragraph
    let footer = Paragraph::new(Text::styled(
        "Press 'Q' to quit, 'TAB' to change units",
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::LightYellow)
            .add_modifier(Modifier::BOLD),
    ))
    .block(footer_block);

    // RENDER STUFF
    f.render_widget(title, chunks[0]);
    f.render_widget(footer, chunks[2]);
    f.render_widget(load, loads_mem[0]);
    f.render_widget(memory, loads_mem[1]);
    f.render_widget(temp, battery_temp_chunks[0]);
    f.render_widget(battery_block,battery_temp_chunks[1]);
    f.render_widget(battery_percent, battery_recs[0]);
    f.render_widget(battery_gauge, battery_recs[1]);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
