/*
- This module should handle the rendering and layout of the thing
*/

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, prelude::Alignment
};

use crate::app::{App, CurrentScreen};

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
    let title = Paragraph::new(
        Text::styled("system stats", Style::default()
            .add_modifier(Modifier::BOLD))
    ).block(title_block)
    .alignment(Alignment::Center);
    
    // SPlit in 2 blocks for info
    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);
    
    let temp_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title("CPU Temperatur");
    
    let load_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title("System Load");
    
    //Quit message box
    let footer_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::LightYellow));
    // Quit paragraph
    let footer = Paragraph::new(
        Text::styled("Press 'Q' to quit",
            Style::default()
            .fg(Color::DarkGray)
            .bg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
        )).block(footer_block);
    // RENDER STUFF
    // Title
    f.render_widget(title, chunks[0]);
    f.render_widget(footer, chunks[2]);

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