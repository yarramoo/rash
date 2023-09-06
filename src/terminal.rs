use console::{self, Key, Term};
use std::io::{self, Write};

use crate::PROMPT_STR;

#[allow(dead_code)]
pub fn test_term_size() -> io::Result<String> {
    let mut term = Term::stdout();
    let terminal_width = term.size().1 as usize;
    let s = "a".repeat(terminal_width + 1);
    term.write_line(&terminal_width.to_string())?;
    term.write_all(s.as_bytes())?;
    Ok(String::new())
}

// Interactive shell that allows users to input commands and tracks the command string as it is being typed
// Use this later to auto suggest based on current unfinished input
pub fn get_cmd_interactive() -> io::Result<String> {
    // Terminal
    let mut term = Term::stdout();
    let terminal_width = term.size().1 as usize;
    // Command String buffer 
    let mut buffer = "m".repeat(terminal_width * 3 - terminal_width / 3);
    let mut cursor_i = buffer.len();

    term.write_all(buffer.as_bytes())?;

    loop {
        // Check if the terminal size changed
        let new_terminal_width = term.size().1 as usize;
        if new_terminal_width != terminal_width {
            // Redraw the command for new width

        }
        let key = term.read_key()?;
        match key {
            Key::Char(c) => write_char(&mut term, &mut buffer, c, &mut cursor_i)?,
            Key::Backspace => {
                if cursor_i == PROMPT_STR.len() { continue; }
                delete_char(&mut term, &mut buffer, &mut cursor_i)?;
            },
            Key::Enter => {
                // Input command
                term.write_line("")?;
                break;
            },
            Key::Tab => {
                // Logic for making regex suggestions goes here
                todo!();
            },
            Key::ArrowRight => {
                // Move right if we are still within the buffer space
                // Two cases. a) we are within a line so we just move right. b) we are on the right edge and must go down a line 
                if cursor_i == buffer.len() { continue; }
                if cursor_i % terminal_width != terminal_width - 1 {
                    term.move_cursor_right(1)?;
                } else {
                    term.move_cursor_down(1)?;
                    term.move_cursor_left(terminal_width)?;
                }
                cursor_i += 1;
            },
            Key::ArrowLeft  => {
                if cursor_i == PROMPT_STR.len() { continue; }
                // Two cases. a) we are within a line so we just move left. b) we are on the left edge of the 
                // line so we must move up and to the right
                if cursor_i % terminal_width != 0 {
                    term.move_cursor_left(1)?;
                } else {
                    term.move_cursor_up(1)?;
                    term.move_cursor_right(terminal_width)?;
                }
                cursor_i -= 1;
            },
            Key::ArrowUp => {
                if cursor_i < terminal_width { continue; }
                if cursor_i - terminal_width < PROMPT_STR.len() {
                    let r = terminal_width - (cursor_i - PROMPT_STR.len());
                    term.move_cursor_right(r)?;
                    cursor_i = PROMPT_STR.len();
                } else {
                    cursor_i -= terminal_width;
                }
                term.move_cursor_up(1)?;
            },
            Key::ArrowDown => {
                if buffer.len() - cursor_i < buffer.len() % terminal_width { continue; }
                if buffer.len() - cursor_i >= terminal_width {
                    cursor_i += terminal_width;
                } else {
                    term.move_cursor_left(terminal_width - (buffer.len() - cursor_i))?;
                    cursor_i = buffer.len();
                }
                term.move_cursor_down(1)?;
            }
            _ => {},
        };
    }
    Ok(buffer.split_off(PROMPT_STR.len()))
}

struct Cursor {
    buf_index: usize,
    terminal_width: usize,
}

impl Cursor {
    fn new(buf_index: usize, term: &Term) -> Self {
        Cursor { buf_index, terminal_width: term.size().1 }
    }

    fn position(&self) -> (usize, usize) {
        (self.buf_index / self.terminal_width, self.buf_index, self.terminal_width)
    }

    fn cursor_position(&self) -> (usize, usize) {
        (self.buf_index / self.terminal_width, self.buf_index % self.terminal_width)
    }

    fn move_relative(&mut self, term: &mut Term, line_relative: isize, col_relative: isize) -> io::Result<()> {
        // Move vertically
        if line_relative > 0 {
            term.move_cursor_down(line_relative as usize)?;
        } else if line_relative < 0 {
            term.move_cursor_down((-line_relative) as usize)?;
        }
        // Move horizontally
        if col_relative > 0 {
            term.move_cursor_right(col_relative as usize)?;
        } else if col_relative < 0 {
            term.move_cursor_left((-col_relative) as usize)?;
        }
        Ok(())
    }
}

/// Find the cursor position (line, col). line measured by lines down from beginning. col measured by characters from the left

/// Find the cursor index from a given line and column
fn cursor_index(terminal_width: usize, line: usize, col: usize) -> usize {
    line * terminal_width + col
}

/// Move the cursor to a target index. Passing terminal width explicitly because the terminal size may have changed
fn move_cursor_given_index(
    term: &mut Term, 
    terminal_width: usize, 
    current_index: usize, 
    target_index: usize
) -> io::Result<()>
{
    let (current_line, current_row) = cursor_position(terminal_width, current_index);
    let (target_line, target_row) = cursor_position(terminal_width, target_index);
    // Move vertically
    if target_line < current_line {
        term.move_cursor_up(current_line - target_line)?;
    } else if target_line > current_line {
        term.move_cursor_down(target_line - current_line)?;
    }
    // Move horizontally
    if target_row < current_row {
        term.move_cursor_left(current_row - target_row)?;
    } else if target_row > current_row {
        term.move_cursor_right(target_row - current_row)?;
    }
    Ok(())
}

fn write_char(term: &mut Term, buffer: &mut String, c: char, cursor_i: &mut usize) -> io::Result<()> {
    let terminal_width = term.size().1 as usize;
    buffer.insert(*cursor_i, c);
    update_terminal(term, buffer, *cursor_i)?;
    *cursor_i += 1;
    term.move_cursor_right(1)?;
    if *cursor_i % terminal_width == 0 {
        term.write_line("")?;
    }
    Ok(())
}


fn delete_char(term: &mut Term, buffer: &mut String, cursor_i: &mut usize) -> io::Result<()> {
    // Remove a character. More cursor up if deleting past start of line 
    let terminal_width = term.size().1 as usize;
    buffer.remove(*cursor_i-1);
    *cursor_i -= 1;
    // Special case when removing the last character in a line
    if *cursor_i % terminal_width == 0 && *cursor_i == buffer.len() {
        term.clear_line()?;
        return Ok(())
    }
    if *cursor_i % terminal_width == terminal_width - 1 {
        term.move_cursor_up(1)?;
        term.move_cursor_right(terminal_width)?;
    }
    update_terminal(term, buffer, *cursor_i)?;
    Ok(())
}


fn update_terminal(term: &mut Term, buffer: &str, cursor_i: usize) -> io::Result<()> {
    // Find the number of lines that need updating
    let prev_cursor_position = cursor_i;
    let terminal_width = term.size().1 as usize;

    let current_line_start_i = cursor_i - cursor_i % terminal_width;

    let chars_after_cursor = buffer.len() - current_line_start_i;
    let lines_after_cursor = (chars_after_cursor + terminal_width - 1) / terminal_width; // ceiling div
    
    // Replace the lines below with updated buffer
    let mut i = current_line_start_i;
    while i < buffer.len() {
        term.clear_line()?;
        let j = (i + terminal_width).min(buffer.len());
        if j < buffer.len() {
            term.write_line(&buffer[i..j])?;
        } else {
            term.write_all(&buffer.as_bytes()[i..j])?;
        }
        i += terminal_width;
    } 
    // Move cursor back to previous position
    move_cursor_given_index(term, terminal_width, buffer.len(), cursor_i)?;
    Ok(())
}

