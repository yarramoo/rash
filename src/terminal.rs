use console::{self, Key, Term};
use std::io::{self, Write};

use crate::PROMPT_STR;

// Interactive shell that allows users to input commands and tracks the command string as it is being typed
// Use this later to auto suggest based on current unfinished input
pub fn get_cmd_interactive() -> io::Result<String> {
    // Make a terminal stdout handle
    let mut term = Term::stdout();
    // Terminal height and width
    let t_width= term.size().1 as usize;
    // String buffer to hold command
    let mut buf = String::from(PROMPT_STR);
    // Indices into the buffer to track the bottom-most line being edited
    let (mut i, mut j) = (0, PROMPT_STR.len());
    // Index of cursor position. Used for cursor movement
    let mut cursor_i = PROMPT_STR.len();
    // Write the prompt to the terminal
    term.write_all(buf.as_bytes())?;
    // Loop on key input
    loop {
        let key = term.read_key()?;
        match key {
            Key::Char(c) => {
                // Add the character. Add a new line if the current terminal line is full.
                // Clear the line and reprint the line with the new character
                buf.push(c);
                j += 1;
                if j - i == t_width - 1 {
                    term.write_line("")?;
                    i = j;
                }
                // term.clear_line()?;
                term.write_all(c.to_string().as_bytes())?;
                // term.write_all(&buf.as_bytes()[i..j])?;
            },
            Key::Backspace => {
                // Delete a character. Reduce the slice of the buffer that is shown.
                // If the current terminal line empties then move the cursor up and to the right to edit the above line
                // Adjust slice into buffer
                if j == PROMPT_STR.len() { continue; }
                let deleted_char = buf.pop();
                j -= 1;
                term.clear_line()?;
                if i == j && i != 0 {
                    term.move_cursor_up(1)?;
                    term.move_cursor_right(t_width)?;
                    i = j - (t_width-1);
                    term.clear_line()?;
                }
                term.write_all(&buf.as_bytes()[i..j])?;
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
            Key::ArrowRight => term.move_cursor_right(1)?,
            Key::ArrowLeft  => term.move_cursor_left(1)?,
            _ => {},
        };
    }
    Ok(buf.split_off(PROMPT_STR.len()))
}