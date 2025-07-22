use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Stylize, Alignment},
    style::{palette::tailwind::GREEN, Color},
    text::{Text, Line},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame
};
use std::{thread::{self}, time};
use rand::{prelude::*, random_range};
use rand::random_bool;

const UPDATE_RATE_MILLIS: u64 = 90;

struct InputLabelGuard {
    original_label: Option<String>,
}

impl InputLabelGuard {
    fn new(state: &mut State, new_label: Option<String>) -> Self {
        let original_label = state.input_label_text.clone();

        if let Some(label) = new_label {
            state.input_label_text = label;
        }
        
        InputLabelGuard {
            original_label: Some(original_label)
        }
    }

    fn restore(mut self, state: &mut State) {
        if let Some(original) = self.original_label.take() { // if self.original_label.take() is Some(), then put it in original
            state.input_label_text = original;
        }
    }
}

impl Drop for InputLabelGuard {
    fn drop(&mut self) {
        if self.original_label.is_some() {
            eprintln!("Warning: InputLabelGuard dropped without explicit restore");
        }
    }
}
struct State {
    menu_items: Vec<String>,
    selected_index: usize,
    input_mode: Option<usize>,
    input_string: String,
    input_label_text: String,
    output_widget_messages: Vec<String>,
}

impl State {
    fn new() -> Self {
        Self {
            menu_items: vec![
                String::from("Coinflip"),
                String::from("Percentage Chance Roll"),
                String::from("Password Generator"),
                String::from("Range Randomization")
            ],
            selected_index: 0,
            input_mode: None,
            input_string: String::new(),
            input_label_text: String::from("Input"), // Shows prompt dialog labeled with "input" by default
            output_widget_messages: Vec::new(),
        }
    }

    fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.menu_items.len() - 1;
        }
    }

    fn select_next(&mut self) {
        if self.selected_index < self.menu_items.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    fn push_message_output(&mut self, msg: String) {
        if self.output_widget_messages.len() > 16 {
            self.output_widget_messages.clear();
        }
        self.output_widget_messages.push(msg);
    }
}

fn prompt_user_input(terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>, state: &mut State, input_label: Option<String>) -> Option<String> {
    state.input_string.clear();
    state.input_mode = Some(state.selected_index);

    let _guard = InputLabelGuard::new(state, input_label);
    loop {
        terminal.draw(|frame| draw(frame, state)).expect("failed to draw frame");
        thread::sleep(time::Duration::from_millis(UPDATE_RATE_MILLIS));

        if let Event::Key(key) = event::read().expect("failed to read event") {
            match key.code {
                // add and remove chars as they are typed
                KeyCode::Char(c) => {
                    state.input_string.push(c);
                }
                KeyCode::Backspace => {
                    state.input_string.pop();
                }

                KeyCode::Enter => {
                    let result = state.input_string.clone();
                    state.input_mode = None;
                    state.input_string.clear();
                    _guard.restore(state);
                    return Some(result);
                }
                KeyCode::Esc => {
                    state.input_mode = None;
                    state.input_string.clear();
                    _guard.restore(state);
                    return None;
                }
                _ => {}
            }
        }
    }
}

fn main() {
    cli_log::init_cli_log!();
    let mut terminal = ratatui::init();
    let mut state: State = State::new();
    let mut rng = rand::rng();

    loop {
        terminal.draw(|frame| draw(frame, &state)).expect("failed to draw frame");
        thread::sleep(time::Duration::from_millis(UPDATE_RATE_MILLIS));

        if state.input_mode.is_none() {
            // only process menu navigation keys when not in input mode
            if let Event::Key(key) = event::read().expect("failed to read event") {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => state.select_previous(),
                    KeyCode::Down => state.select_next(),
                    KeyCode::Enter => {
                        match state.selected_index {
                            0 => { // coinflip
                                state.push_message_output(rng.random_bool(0.5).to_string());
                            }
                            1 => { // percent chance roll
                                if let Some(input) = prompt_user_input(&mut terminal, &mut state, Some(String::from("Enter percentage"))) {
                                    match input.trim().parse::<f64>() {
                                        Ok(value) => {
                                            let pvalue = value / 100.0;
                                            state.push_message_output(format!("Hit: {}", rng.random_bool(pvalue)));
                                        }
                                        Err(error) => {
                                            state.push_message_output(format!("ERROR: {}", error));
                                        }
                                    }
                                } else {
                                    state.push_message_output("input cancelled".to_string());
                                }
                            }
                            2 => { // password generator
                                let mut generated_password = String::new();
                                let mut chartable: Vec<char> = vec![
                                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                                    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
                                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
                                    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~'
                                ];
                                chartable.shuffle(&mut rng);
                                if let Some(input) = prompt_user_input(&mut terminal, &mut state, Some(String::from("Enter length of password"))) {
                                    match input.trim().parse::<usize>() {
                                        Ok(length) => {
                                            if length > chartable.len() {
                                                state.push_message_output(format!("You've entered a password length too long"));
                                            } else {
                                                for c in 0..length {
                                                    generated_password.push(chartable[c]);
                                                }
                                                state.push_message_output(format!("randomized password: {}", generated_password));
                                            }
                                        }
                                        Err(error) => {
                                            state.push_message_output(format!("ERROR: {}", error));
                                        }
                                    }
                                }
                                else {
                                    state.push_message_output("input cancelled".to_string());
                                }
                            }
                            3 => { // range randomization
                                if let Some(input) = prompt_user_input(&mut terminal, &mut state, Some(String::from("Enter maximum number of range"))) {
                                    match input.trim().parse::<u64>() {
                                        Ok(max_int) => {
                                            let answer:u64 = rng.random_range(1..=max_int);
                                            state.push_message_output(format!("Answer : {}", answer));
                                        }
                                        Err(error) => {
                                            state.push_message_output(format!("ERROR: {}", error));
                                        }
                                    }
                                }
                            }
                            _ => state.push_message_output("Severe error".to_string()),
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame, state: &State) {
    let outer_layout = Layout::default()
        .margin(1)
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    let menu_lines: Vec<Line> = state.menu_items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == state.selected_index {
                Line::from(format!("> {}", item))
                    .style(Color::from(GREEN.c300))
                    .bold()
            } else {
                Line::from(format!("  {}", item))
            }
        })
        .collect();

    let menu_block = Block::new().title("Menu").borders(Borders::ALL);
    let menu_widget = Paragraph::new(Text::from(menu_lines))
        .block(menu_block)
        .alignment(Alignment::Center);

    if state.input_mode.is_some() {
        let input_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(outer_layout[0]);
        frame.render_widget(menu_widget, input_layout[0]);

        let input_text = format!("{}: {}_", state.input_label_text, state.input_string);
        let input_widget = Paragraph::new(input_text)
            .block(Block::new().title("Input Prompt").borders(Borders::ALL));
        frame.render_widget(input_widget, input_layout[1]);
    } else {
        frame.render_widget(menu_widget, outer_layout[0]);
    }

    let output_list: Vec<ListItem> = state.output_widget_messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    frame.render_widget(
        List::new(output_list)
            .block(Block::new().title("Output").borders(Borders::ALL)),
        outer_layout[1],
    );
}
