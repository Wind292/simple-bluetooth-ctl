mod bluetooth;
use bluetooth::Device;

use color_eyre::Result;
use btleplug::platform::Peripheral;
use color_eyre::eyre::Ok;
use color_eyre::owo_colors::OwoColorize;
use ratatui::text::Span;
use tokio;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::ListItem;
use ratatui::widgets::List;
use ratatui::style::{Color, Style};

use std::sync::Arc;
use std::{i32, usize};
use tokio::sync::Mutex;
use std::time::Duration;






use crate::bluetooth::Instance;
#[tokio::main]
async fn main() -> Result<()> {
    let mut inst = bluetooth::Instance::new();
    inst.init().await;
    inst.start_scan().await;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, inst).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal, inst: Instance) -> Result<()> {

    let devices_mutex: Arc<Mutex<Vec<Device>>> = Arc::new(Mutex::new(vec![]));
    let instance_mutex: Arc<Mutex<Instance>> = Arc::new(Mutex::new(inst));
    let satus_mutex: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));

    let devices_refrence = Arc::clone(&devices_mutex);
    let instance_refrence = Arc::clone(&instance_mutex);
    let status_refrence= Arc::clone(&satus_mutex);

    let mut popup_open: bool = false;
    let mut is_selected = false;
    let mut selection_index: i32 = -1;
    let mut selection_index_x = 0;

    tokio::spawn(async move { background_device_update(devices_refrence, instance_refrence, status_refrence).await; });
    loop {
        tokio::time::sleep(Duration::from_millis(5)).await; // give time for the background thread to pick up the Mutex
        let instance = instance_mutex.lock().await;
        let devices = devices_mutex.lock().await;

        selection_index = selection_index.clamp(-1, (devices.len()-1) as i32);


        terminal.draw(|frame| render(frame, &devices, &instance, selection_index, is_selected, popup_open))?;

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(KeyEvent { code, modifiers, kind, state }) = event::read()? {
                match code {
                    KeyCode::Up => {if selection_index != 0 && !popup_open {selection_index -= 1} is_selected = true; popup_open = false;}
                    KeyCode::Down => {if selection_index != (devices.len()-1) as i32 {selection_index += 1} is_selected =true; popup_open = false;}
                    KeyCode::Right => {selection_index_x+=1; popup_open = true; is_selected = false}   
                    KeyCode::Left => {selection_index_x-=1; popup_open = true; is_selected = false}
                    KeyCode::Enter => {popup_open= true; is_selected=true}
                    KeyCode::Esc => break Ok(()),
                    _=>{}
                }
            }
        }


    }
}

fn render(frame: &mut Frame<'_>, devices: &Vec<Device>, instance: &Instance, mut selection_index: i32, is_selected: bool, popup_open: bool) {
    // Convert strings into ListItems
    let mut list_items: Vec<ListItem> = devices.clone()
        .iter()
        .enumerate()
        .map(|(i, s)| ListItem::new(format_line(s.adress.clone(), s.display_name.clone(), s.connection_strength, frame.area().width.into(), Some(i), selection_index as usize, popup_open, is_selected)))
        .collect();

        
    if popup_open {
        list_items.insert((selection_index+1) as usize, ListItem::new(format_dropdown(frame.area().width.into())));
    }
    

    // Create a List with a border
    let list = List::new(list_items)
        .block(
            Block::default()
                .title("Devices")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    frame.render_widget(list, frame.area());


}

fn format_dropdown<'a>(term_width: u64) -> Span<'a> {
    let mut text = "    ╰─ (c) Connect".to_string();

    if term_width > text.len() as u64 {
        for i in 0..term_width-text.len() as u64 {
            text.push(' ');
        } 

    }

    Span::styled(text, Style::default().bg(Color::Cyan))
}

fn format_line<'a>(adress: String, name: Option<String>, connection_strength: Option<i16>, term_width: u64, index: Option<usize>, selection_index: usize, popup_open: bool, is_selected: bool) -> Span<'a> {
    let mut line = format!(" {:?} - {}, {:?}", name, adress, connection_strength);

    if term_width > line.len() as u64 {
        for i in 0..term_width-line.len() as u64 {
            line.push(' ');
        } 

    }

    if selection_index == index.unwrap_or(usize::MAX) && !popup_open && is_selected {
        return Span::styled(line, Style::default().bg(Color::Cyan));
    }
   
    Span::styled(line, Style::default())
}

async fn background_device_update(devices_mutex: Arc<Mutex<Vec<Device>>>, instance_mutex: Arc<Mutex<Instance>>, status_mutex: Arc<Mutex<bool>>) {
    loop {
        if *status_mutex.lock().await {
            let mut instance = instance_mutex.lock().await;
            let updated_devices = instance.get_scanned_devices().await;
            let mut devices = devices_mutex.lock().await;
            *devices = updated_devices.clone();
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}