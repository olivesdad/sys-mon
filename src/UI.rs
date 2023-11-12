/*
- This module should handle the rendering and layout of the thing
*/

use crate::app::{App, Units};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Text},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, GraphType, Padding, Paragraph, Wrap,
    },
    Frame,
};

// Main function used to render the UI.
// passed as a closure to the draw function which passes the frame size to it.
// We also pass an app reference to it so we can query the state of hte world
pub fn ui(f: &mut Frame, app: &App) {
    // If the Frame is too small just render warning so that we dont panic
    let w = f.size();
    if w.width < 70 || w.height < 15 {
        let rect = centered_rect(f.size(), 50, 50);
        let warning = Paragraph::new(Text::styled("Window is too small 😵", Style::default()))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        // Block
        let block = Block::default()
            .borders(Borders::all())
            .title(format!(" Width: {}, Height: {} ", &w.width, w.height));
        //innter rect
        let inner = centered_rect(block.inner(rect), 50, 50);

        // Render
        f.render_widget(block, rect);
        f.render_widget(warning, inner);
        return;
    }

    // Start main screen here vvvvv

    // Sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    ////////////// Title Box///////////////
    // Border box thing
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(" OOga Booga ");
    // Paragraph widget takes ownership of title_block
    let title = Paragraph::new(Text::styled(
        "system stats",
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .block(title_block)
    .alignment(Alignment::Center);

    //-----------------------------------------------------//
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

    //////  +++++++++++ Battery Block ++++++++++++++ ////////
    let battery_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(" Battery 🔋 ");
    // Split again
    let battery_space = battery_block.inner(battery_temp_chunks[1]);
    let battery_recs = Layout::default()
        .constraints([Constraint::Max(3), Constraint::Min(4)])
        .split(battery_space);

    // Battery widget Paragraph
    let battery_percent = Paragraph::new(Text::styled(app.get_battery_time(), Style::default()))
        .alignment(Alignment::Center);

    // Battery Gauge Widget
    let battery_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(app.get_battery_color())
                .bg(Color::DarkGray),
        )
        .percent(app.get_battery_left() as u16);

    // ++++++++++++ CPUT TEMP BLOCK + PARAGRAPH ++++++++++++ //
    let temp_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(" CPU Temperature 🔥 ");
    // unit char
    let unit = match app.units {
        Units::Celcius => "C",
        Units::Farenheight => "F",
    };

    let temp = Paragraph::new(Text::styled(
        app.get_temp().to_string() + unit,
        Style::default(),
    ))
    .alignment(Alignment::Center);

    // CHART FOR TEMP
    // DATATSET
    let dataset = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::LightBlue))
        .data(app.get_temp_points())];
    let chart = Chart::new(dataset)
        .x_axis(
            Axis::default()
                .title("")
                .style(Style::default())
                .bounds([0.0, app.get_temp_points().len() as f64]),
        )
        .y_axis(
            Axis::default()
                .title("Temp (C)")
                .style(Style::default())
                .bounds([0.0, 120.0])
                .labels(
                    ["0", "20", "40", "60", "80", "100", "120"]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        );

    let temp_inner = temp_block.inner(battery_temp_chunks[0]);
    let temp_chunks = Layout::default()
        .constraints([Constraint::Max(3), Constraint::Min(10)])
        .direction(Direction::Vertical)
        .split(temp_inner);

    // +++++++ CPU LOAD BLOCK + PARAGRAPH  ++++++++ //

    // BLOCK FOR LOADS TO BE RENDERED IN >>>>> battery_temp_chunks[1]
    let load_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(" System Load (%)🏋️  ");

    let load_bars_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default())
        .padding(Padding::new(3, 3, 1, 1));

    // Barchart loadas
    let loads = app.get_load();
    let load_bars = BarChart::default()
        .data(&[
            ("nice", *loads.get("nice").unwrap() as u64),
            ("user", *loads.get("user").unwrap() as u64),
            ("system", *loads.get("system").unwrap() as u64),
            ("interrupt", *loads.get("interrupt").unwrap() as u64),
            ("idle", *loads.get("idle").unwrap() as u64),
        ])
        .bar_width(5)
        .max(100)
        .block(load_bars_block);

    // Split battery chunks 1 to center chart

    // I need the width to find center
    let w = loads_mem[0].width;
    let load_bars_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max((&w - 35) / 2),
            Constraint::Min(35),
            Constraint::Max((&w - 35) / 2),
        ])
        .split(loads_mem[0]);
    // ++++++++ MEMORY USAGE BLOCK -++++++++//
    let mem_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(" Memory Usage 🧠 ");

    let (x, y) = app.get_mem();
    let memory = Paragraph::new(Text::styled(
        format!("{} Used / {} Total", x, y),
        Style::default(),
    ))
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
    f.render_widget(load_block, loads_mem[0]);
    f.render_widget(load_bars, load_bars_chunks[1]);
    f.render_widget(memory, loads_mem[1]);
    f.render_widget(temp_block, battery_temp_chunks[0]);
    f.render_widget(chart, temp_chunks[1]);
    f.render_widget(temp, temp_chunks[0]);
    f.render_widget(battery_block, battery_temp_chunks[1]);
    f.render_widget(battery_percent, battery_recs[0]);
    f.render_widget(battery_gauge, battery_recs[1]);
}

/// # Usage
///
/// ```rust
/// let rect = centered_rect(f.size(), 50, 50);
/// ```
fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
