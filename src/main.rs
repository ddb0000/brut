use std::fs::File;
use std::io;
use std::io::Write;
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ropey::Rope;

struct Editor {
    should_quit: bool,
    rope: Rope,
    cursor_line: usize,
    cursor_col: usize,
    filename: Option<String>,
    show_save_prompt: bool,
    is_new_file: bool,
    input_buffer: String,
    preferred_column: usize,
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
            cursor_line: 0,
            cursor_col: 0,
            filename: filename.clone(),
            show_save_prompt: false,
            is_new_file,
            input_buffer: String::new(),
            preferred_column: 0,
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

    fn cursor_screen_position(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_line)
    }

    fn cursor_char_index(&self) -> usize {
        self.rope.line_to_char(self.cursor_line) + self.cursor_col
    }

    fn line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.rope.len_lines() {
            return 0;
        }
        let line = self.rope.line(line_idx);
        let len = line.len_chars();
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    fn handle_prompt_key(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(c) => {
                if self.is_new_file {
                    self.input_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                }
            }
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() || !self.is_new_file {
                    self.save();
                }
                self.should_quit = true;
                self.show_save_prompt = false;
            }
            KeyCode::Esc => {
                self.should_quit = true;
                self.show_save_prompt = false;
            }
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Enter => {
                let index = self.cursor_char_index();
                self.rope.insert(index, "\n");
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.preferred_column = self.cursor_col;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let index = self.cursor_char_index();
                    self.rope.remove(index - 1..index);
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    let index = self.cursor_char_index();
                    self.rope.remove(index - 1..index);
                    self.cursor_line -= 1;
                    self.cursor_col = self.line_len(self.cursor_line);
                }
                self.preferred_column = self.cursor_col;
            }
            KeyCode::Esc => {
                self.show_save_prompt = true;
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => {
                let index = self.cursor_char_index();
                self.rope.insert(index, &c.to_string());
                self.cursor_col += 1;
                self.preferred_column = self.cursor_col;
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.line_len(self.cursor_line);
                }
                self.preferred_column = self.cursor_col;
            }
            KeyCode::Right => {
                if self.cursor_char_index() < self.rope.len_chars() {
                    let line_len = self.line_len(self.cursor_line);
                    if self.cursor_col < line_len {
                        self.cursor_col += 1;
                    } else if self.cursor_line + 1 < self.rope.len_lines() {
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                    }
                }
                self.preferred_column = self.cursor_col;
            }
            KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    let line_len = self.line_len(self.cursor_line);
                    self.cursor_col = std::cmp::min(self.preferred_column, line_len);
                }
            }
            KeyCode::Down => {
                if self.cursor_line + 1 < self.rope.len_lines() {
                    self.cursor_line += 1;
                    let line_len = self.line_len(self.cursor_line);
                    self.cursor_col = std::cmp::min(self.preferred_column, line_len);
                }
            }
            _ => {}
        }
    }

    fn run(&mut self) {
        let mut last_tick = Instant::now();
        loop {
            let timeout = Duration::from_millis(200)
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));

            if self.show_save_prompt {
                if let CEvent::Key(key_event) = event::read().unwrap() {
                    self.handle_prompt_key(key_event.code);
                }
            } else if crossterm::event::poll(timeout).unwrap() {
                if let CEvent::Key(key_event) = event::read().unwrap() {
                    self.handle_editor_key(key_event.code);
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
            let (cursor_x, cursor_y) = self.cursor_screen_position();
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
