use std::io;
use std::io::Write;
use std::fs::File;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use crossterm::cursor::{Hide, Show};
use crossterm::cursor::MoveTo;
use crossterm::execute;

use ropey::Rope;

enum Event<I> {
    Input(I),
    Tick,
}

struct Editor {
    should_quit: bool,
    rope: Rope,
    cursor_position: (usize, usize),
    filename: Option<String>,
    show_save_prompt: bool,
    is_new_file: bool,
    input_buffer: String,
    preferred_column_position: usize,

}

impl Editor {
    fn open(filename: Option<String>) -> Self {
        let mut rope = Rope::new();
        let is_new_file = filename.is_none();
        if let Some(ref file) = filename {
            if let Ok(contents) = std::fs::read_to_string(file) {
                rope = Rope::from_str(&contents);
            }
        }
        Self {
            should_quit: false,
            rope,
            cursor_position: (0, 0),
            filename: filename.clone(),
            show_save_prompt: false,
            is_new_file,
            input_buffer: String::new(),
            preferred_column_position: 0,
        }
    }    

    fn save(&mut self) {
        let filename = if self.filename.is_some() {
            self.filename.as_ref().unwrap()
        } else {
            &self.input_buffer
        };
    
        let mut file = File::create(filename).expect("Could not create file");
        write!(file, "{}", self.rope).expect("Could not write to file");
    
        if self.is_new_file {
            self.filename = Some(filename.to_string());
            self.is_new_file = false;
        }
    }

    fn calculate_cursor_position(&self) -> (usize, usize) {
        let mut cursor_x = 0;
        let mut cursor_y = 0;
        for (i, char) in self.rope.chars().enumerate() {
            if i >= self.cursor_position.1 {
                break;
            }
            if char == '\n' {
                cursor_y += 1;
                cursor_x = 0;
            } else {
                cursor_x += 1;
            }
        }
        (cursor_x, cursor_y)
    }
    
    
    fn run(&mut self) {
        let mut last_tick = Instant::now();
        loop {
            let timeout = Duration::from_millis(200)
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));
    
            if self.show_save_prompt {
                if let CEvent::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            if self.is_new_file && self.show_save_prompt {
                                self.input_buffer.push(c);
                                self.draw();
                            }
                        },
                        KeyCode::Backspace => {
                            if !self.input_buffer.is_empty() {
                                self.input_buffer.pop();
                            }
                        },       
                        KeyCode::Enter => {
                            if !self.input_buffer.is_empty() || !self.is_new_file {
                                self.save();
                            }
                            self.should_quit = true;
                            self.show_save_prompt = false;
                        },
                        KeyCode::Esc => {
                            self.should_quit = true;
                            self.show_save_prompt = false;
                        },
                        _ => {}
                    }
                }              
            } else {
                if crossterm::event::poll(timeout).unwrap() {
                    if let CEvent::Key(key_event) = event::read().unwrap() {
                        match key_event.code {
                            KeyCode::Enter => {
                                self.rope.insert(self.cursor_position.1, "\n");
                                self.cursor_position.0 += 1;
                                self.cursor_position.1 = self.rope.line_to_char(self.cursor_position.0);
                                self.draw();
                            },                            
                            KeyCode::Backspace => {
                                if self.cursor_position.1 > 0 {
                                    self.rope.remove(self.cursor_position.1 - 1..self.cursor_position.1);
                                    self.cursor_position.1 -= 1;
                                }
                            },
                            KeyCode::Esc => {
                                self.show_save_prompt = true;
                                self.draw();
                            },
                            KeyCode::Char(c) => {
                                self.rope.insert(self.cursor_position.1, &c.to_string());
                                self.cursor_position.1 += 1;
                                self.preferred_column_position = self.cursor_position.1 - self.rope.line_to_char(self.cursor_position.0);
                            },
                            KeyCode::Left => {
                                if self.cursor_position.1 > 0 {
                                    self.cursor_position.1 -= 1;
                                    self.preferred_column_position = self.cursor_position.1 - self.rope.line_to_char(self.cursor_position.0);
                                }
                            },
                            KeyCode::Right => {
                                if self.cursor_position.1 < self.rope.len_chars() {
                                    self.cursor_position.1 += 1;
                                    self.preferred_column_position = self.cursor_position.1 - self.rope.line_to_char(self.cursor_position.0);
                                }
                            },
                            KeyCode::Up => {
                                if self.cursor_position.0 > 0 {
                                    self.cursor_position.0 -= 1;
                                    let line_start = self.rope.line_to_char(self.cursor_position.0);
                                    let line_len = self.rope.line(self.cursor_position.0).len_chars();
                                    self.cursor_position.1 = std::cmp::min(line_start + self.preferred_column_position, line_start + line_len);
                                }
                            },
                            KeyCode::Down => {
                                if self.cursor_position.0 < self.rope.len_lines() - 1 {
                                    self.cursor_position.0 += 1;
                                    let line_start = self.rope.line_to_char(self.cursor_position.0);
                                    let line_len = self.rope.line(self.cursor_position.0).len_chars();
                                    self.cursor_position.1 = std::cmp::min(line_start + self.preferred_column_position, line_start + line_len);
                                }
                            },
                            _ => {} 
                        }                    
                    }
                }
            }
            if last_tick.elapsed() >= Duration::from_millis(200) {
                last_tick = Instant::now();
            }
    
            if self.should_quit {
                break;
            }
    
            self.draw();
        }
    }
    
    fn draw(&self) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    if self.show_save_prompt {
        print!("Filename: {}", self.input_buffer);
        print!("\nPress Enter to save or Esc to discard changes.\n");
        let prompt_length = "Filename: ".len();
        let cursor_pos = self.input_buffer.len() + prompt_length;
        execute!(io::stdout(), MoveTo(cursor_pos as u16, 0)).unwrap();
    } else {
        for line in self.rope.lines() {
            print!("{}", line);
        }
        let (cursor_x, cursor_y) = self.calculate_cursor_position();
        execute!(io::stdout(), MoveTo(cursor_x as u16, cursor_y as u16)).unwrap();
    }
    io::stdout().flush().unwrap();
}

}

fn main() {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();

    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).cloned();
    
    let mut editor = Editor::open(filename);
    editor.run();

    terminal::disable_raw_mode().unwrap();
    stdout.execute(LeaveAlternateScreen).unwrap();
}
