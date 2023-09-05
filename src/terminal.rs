use console::{self, Key, Term};
use std::io::{self, Write, BufRead};

use crate::PROMPT_STR;

pub fn test_term_size() -> io::Result<String> {
    let mut term = Term::stdout();
    let terminal_width = term.size().1 as usize;
    let s = "a".repeat(terminal_width + 1);
    term.write_line(&terminal_width.to_string())?;
    term.write_all(&s.as_bytes())?;
    Ok(String::new())
}

// Interactive shell that allows users to input commands and tracks the command string as it is being typed
// Use this later to auto suggest based on current unfinished input
pub fn get_cmd_interactive() -> io::Result<String> {
    // Terminal
    let mut term = Term::stdout();
    let terminal_width = term.size().1 as usize;
    // Command String buffer 
    let mut buffer = String::from(PROMPT_STR);
    let (mut buf_window_start_i, mut buf_window_end_i) = (0, PROMPT_STR.len());
    let mut cursor_i = PROMPT_STR.len();

    term.write_all(buffer.as_bytes())?;

    loop {
        let key = term.read_key()?;
        match key {
            Key::Char(c) => {
                buffer.insert(cursor_i, c);
                update_terminal(&mut term, &buffer, cursor_i);
                cursor_i += 1;
                term.move_cursor_right(1)?;
                if cursor_i % terminal_width == 0 {
                    term.write_line("")?;
                    term.move_cursor_down(1)?;
                }
            },
            Key::Backspace => {
                // Delete a character. Reduce the slice of the buffer that is shown.
                // If the current terminal line empties then move the cursor up and to the right to edit the above line
                // Adjust slice into buffer
                if buf_window_end_i == PROMPT_STR.len() { continue; }
                let deleted_char = buffer.pop();
                if buf_window_start_i == buf_window_end_i {
                    term.move_cursor_up(1)?;
                    term.move_cursor_right(terminal_width)?;
                    buf_window_start_i = buf_window_end_i - terminal_width;
                }                
                buf_window_end_i -= 1;
                cursor_i -= 1;
                term.clear_line()?;
                term.write_all(&buffer.as_bytes()[buf_window_start_i..buf_window_end_i])?;
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
                if cursor_i == buf_window_end_i { continue; }
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
            _ => {},
        };
    }
    Ok(buffer.split_off(PROMPT_STR.len()))
}


fn update_terminal(term: &mut Term, buffer: &str, cursor_i: usize) -> io::Result<()> {
    // Find the number of lines that need updating
    let prev_cursor_position = cursor_i;
    let terminal_width = term.size().1 as usize;
    let current_line_start_i = cursor_i - cursor_i % terminal_width;
    let chars_after_cursor = buffer.len() - current_line_start_i;
    let lines_after_cursor = chars_after_cursor / terminal_width
        + if chars_after_cursor % terminal_width == 0 {0} else {1};
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
    term.move_cursor_up(lines_after_cursor - 1);
    term.move_cursor_left(terminal_width);
    term.move_cursor_right(prev_cursor_position % terminal_width);
    Ok(())
}

