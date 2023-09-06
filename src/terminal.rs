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


fn write_char(term: &mut Term, buffer: &mut String, c: char, cursor_i: &mut usize) -> io::Result<()> {
    let terminal_width = term.size().1 as usize;
    let mut cursor_i = *cursor_i;
    buffer.insert(cursor_i, c);
    update_terminal(term, buffer, cursor_i)?;
    cursor_i += 1;
    term.move_cursor_right(1)?;
    if cursor_i % terminal_width == 0 {
        term.write_line("")?;
    }
    Ok(())
}


fn delete_char(term: &mut Term, buffer: &mut String, cursor_i: &mut usize) -> io::Result<()> {
    // Remove a character. More cursor up if deleting past start of line 
    let terminal_width = term.size().1 as usize;
    let mut cursor_i = *cursor_i;
    buffer.remove(cursor_i-1);
    cursor_i -= 1;
    // Special case when removing the last character in a line
    if cursor_i % terminal_width == 0 && cursor_i == buffer.len() {
        term.clear_line()?;
        return Ok(())
    }
    if cursor_i % terminal_width == terminal_width - 1 {
        term.move_cursor_up(1)?;
        term.move_cursor_right(terminal_width)?;
    }
    update_terminal(term, buffer, cursor_i)?;
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
    if lines_after_cursor > 0 {
        term.move_cursor_up(lines_after_cursor-1)?;
    }
    term.move_cursor_left(terminal_width)?;
    term.move_cursor_right(prev_cursor_position % terminal_width)?;
    Ok(())
}

