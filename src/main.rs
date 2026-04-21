use std::{io, fs, env};
use std::path::{Path, PathBuf};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    crossterm::terminal::SetSize,
    crossterm::execute,
    style::{Stylize, Color, Style},
    widgets::{Block, Paragraph, List, ListItem, ListState},
    layout::{Constraint, Layout},
    DefaultTerminal,
};

fn dir_movement(input: &str, dir: &Path) -> PathBuf {
    if Path::new(input).is_absolute(){
        PathBuf::from(input)
    } else if input == ".." || input == "../" {
        let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()));
        let mut temp = dir.to_path_buf();
        if temp != home {
          temp.pop();
        }
        temp
    } else {
        let full_path = dir.join(input);
        let metadata = fs::metadata(dir.join(input));
        if metadata.map(|m| m.is_dir()).unwrap_or(false){
          full_path
        } else {
          dir.to_path_buf()
        }
    }
}

fn get_entries(path: PathBuf) -> Vec<String> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata(){
                if let Some(name) = entry.file_name().to_str() {
                    if metadata.is_dir(){
                        dirs.push(format!("📁 {}", name));
                    } else {
                        files.push(format!("📄 {}", name));
                    }
                }
            }
        }
    }
    dirs.sort();
    files.sort();
    dirs.extend(files);
    dirs
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    execute!(io::stdout(), SetSize(100, 110))?;
    terminal.clear()?;
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut last_key = String::from("Press any key!");
    let mut current_dir = env::current_dir()?;
    let current_dir_cl = current_dir.clone();
    let mut dirs = get_entries(current_dir_cl);
    let mut index = 0;
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        let msg = last_key.clone();
	      let current_path_text = current_dir.to_str().unwrap_or("").to_string();

        terminal.draw(|frame| {
         
          let block = Block::bordered().border_style(Style::default().fg(Color::Yellow));
          let inner = block.inner(frame.area());
          frame.render_widget(block, frame.area());

          let areas = Layout::vertical([
              Constraint::Length(1),
              Constraint::Length(1),
              Constraint::Length(1),
              Constraint::Length(1),
              Constraint::Length(1),
              Constraint::Min(1),
          ]).split(inner);

          let greeting = Paragraph::new("Hello, Ratatui! 🐭")
              .centered()
              .yellow();

          let movement = Paragraph::new(msg)
              .centered()
              .white();
          
          let dir_label = Paragraph::new("----- Directories -----")
              .centered()
              .style(Style::default().fg(Color::Cyan));

          let dir_path_label = Paragraph::new(current_path_text)
              .centered()
              .style(Style::default().fg(Color::Yellow));

          let items: Vec<ListItem> = dirs
              .iter()
              .map(|d| ListItem::new(d.as_str()).style(Style::default().fg(Color::Green)))
              .collect();

          let dir_list = List::new(items)
              .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

          frame.render_widget(greeting, areas[0]);
          frame.render_widget(movement, areas[1]);
          frame.render_widget(dir_label, areas[2]);
          frame.render_widget(dir_path_label, areas[3]);
          frame.render_stateful_widget(dir_list, areas[5], &mut list_state);
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press{
              match key.code {
                  KeyCode::Char('q') => return  Ok(()) ,
                  KeyCode::Up => {
                    last_key = String::from("Up!");
                    index = (index + dirs.len() - 1) % dirs.len();
                    list_state.select(Some(index));
                  }
                  KeyCode::Down => {
                    last_key = String::from("Down!");
                    index = (index + 1) % dirs.len();
                    list_state.select(Some(index));
                  }
                  KeyCode::Left => { 
                     last_key = String::from("Left!");
                     let read_dir = dir_movement("..", &current_dir);
                     current_dir = read_dir.clone();
                     dirs = get_entries(read_dir);
                     index = 0;
                     list_state.select(Some(index));
                  },
                  KeyCode::Right => {
                    last_key = String::from("Right!");
                    let folder_name = dirs[index]
                      .splitn(2, " ")
                      .nth(1)
                      .unwrap_or(&dirs[index]);
                    let read_dir = dir_movement(folder_name, &current_dir);
                    current_dir = read_dir;
                    dirs = get_entries(current_dir.clone());
                    index = 0;
                    list_state.select(Some(index));
                  }
                  _ => {}
              }
            }
        }
    }
}
