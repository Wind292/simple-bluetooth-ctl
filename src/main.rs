mod bluetooth;
use color_eyre::Result;
use btleplug::platform::Peripheral;
use color_eyre::eyre::Ok;
use tokio;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::ListItem;
use ratatui::widgets::List;
use ratatui::style::{Color, Style};
use std::sync::Arc;
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

    let devices_mutex: Arc<Mutex<Vec<(Peripheral, Option<i16>, Option<String>, String)>>> = Arc::new(Mutex::new(vec![]));
    let instance_mutex: Arc<Mutex<Instance>> = Arc::new(Mutex::new(inst));

    let devices_refrence = Arc::clone(&devices_mutex);
    let instance_refrence = Arc::clone(&instance_mutex);


    tokio::spawn(async move { background_device_update(devices_refrence, instance_refrence).await; });
    loop {
        tokio::time::sleep(Duration::from_millis(5)).await; // give time for the background thread to pick up the Mutex
        let instance = instance_mutex.lock().await;
        let devices = devices_mutex.lock().await;

        terminal.draw(|frame| render(frame, &devices, &instance))?;
        // if matches!(event::read()?, Event::Key(_)) {
        //     break Ok(());
        // }
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(_) = event::read()? {
                break Ok(());
            }
        }

    }
}

fn render(frame: &mut Frame<'_>, devices: &Vec<(Peripheral, Option<i16>, Option<String>, String)>, instance: &Instance) {
    // Convert strings into ListItems
    let list_items: Vec<ListItem> = devices.clone()// unwraping the mutex
        .iter()
        .map(|s| ListItem::new(format_line(s.3.clone(), s.2.clone(), s.1, 100)))
        .collect();

    // Create a List with a border
    let list = List::new(list_items)
        .block(
            Block::default()
                .title("Fruits")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    frame.render_widget(list, frame.area());


}

fn format_line(adress: String, name: Option<String>, connection_strength: Option<i16>, term_width: u64) -> String {
   format!("{:?} - {}", name, adress) 
}

async fn background_device_update(devices_mutex: Arc<Mutex<Vec<(Peripheral, Option<i16>, Option<String>, String)>>>, instance_mutex: Arc<Mutex<Instance>>) {
    loop {
        let mut instance = instance_mutex.lock().await;
        let updated_devices = instance.get_scanned_devices().await;
        let mut devices = devices_mutex.lock().await;
        *devices = updated_devices.clone();
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}