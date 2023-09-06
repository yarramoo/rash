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
    let mut buffer = "m".repeat(terminal_width - terminal_width / 10);
    // let mut buffer = String::from(PROMPT_STR);
    let mut cursor_i = buffer.len();
    let mut cursor = Cursor::new(buffer.len(), terminal_width);

    term.write_all(buffer.as_bytes())?;

    loop {
        // Check if the terminal size changed
        let new_terminal_width = term.size().1 as usize;
        if new_terminal_width != terminal_width {
            // Redraw the command for new width

        }
        let key = term.read_key()?;
        match key {
            Key::Char(c) => write_char(&mut term, &mut buffer, c, &mut cursor)?,
            Key::Backspace => {
                if cursor.buf_index == PROMPT_STR.len() { continue; }
                delete_char(&mut term, &mut buffer, &mut cursor)?;
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

// How do we interact with the terminal? We type in characters, we move around. It would be nice to interact with a 
// Cursor object when moving around that can track the position and indices for us. 
// When we type, we expect the cursor to move over one. When we delete, we expect the cursor to move back one.
// When move hit the arrow keys we expect the cursor to jump around.
// When we resize we expect the cursor to update its position but keep its index

// IDEA implement everything as a jump forward or jump back. Maybe just a jump with isize argument

impl Cursor {
    fn new(buf_index: usize, terminal_width: usize) -> Self {
        Cursor { buf_index, terminal_width }
    }

    fn cursor_position(&self) -> (usize, usize) {
        (self.buf_index / self.terminal_width, self.buf_index % self.terminal_width)
    }

    fn update_index(&mut self, jump_length: isize) {
        let new_index = self.buf_index as isize + jump_length;
        if new_index < 0 {
            panic!("{}", format!("Trying to jump to negative index. Current index: {}, jump length: {}", self.buf_index, jump_length));
        }
        self.buf_index = (self.buf_index as isize + jump_length) as usize;
    }

    fn jump_checked(&mut self, term: &mut Term, mut jump_length: isize, index_lb: usize, index_ub: usize) -> io::Result<()> {
        // Adjust jump size to be bounded
        if self.buf_index as isize + jump_length > index_ub as isize {
            jump_length = index_ub as isize - self.buf_index as isize;
        } else if self.buf_index as isize + jump_length < index_lb as isize {
            jump_length = index_lb as isize - self.buf_index as isize;
        }
        self.jump(term, jump_length)?;
        Ok(())
    }

    fn jump(&mut self, term: &mut Term, mut jump_length: isize) -> io::Result<()> {
        // Get current and target coordinates
        let (cur_line, cur_col) = self.cursor_position();
        // dbg!(cur_line);
        // dbg!(cur_col);
        self.update_index(jump_length);
        let (target_line, target_col) = self.cursor_position();
        // dbg!(target_line);
        // dbg!(target_col);
        // Move cursor in terminal
        self.move_relative(
            term,
            target_line as isize - cur_line as isize,
            target_col  as isize - cur_col as isize
        )?;
        Ok(())
    }

    fn move_relative(&mut self, term: &mut Term, line_relative: isize, col_relative: isize) -> io::Result<()> {
        // Move vertically
        if line_relative > 0 {
            term.move_cursor_down(line_relative as usize)?;
            // println!("Moving down {line_relative}");
        } else if line_relative < 0 {
            term.move_cursor_up((-line_relative) as usize)?;
            // println!("Moving up {}", -line_relative);
        }
        // Move horizontally
        if col_relative > 0 {
            term.move_cursor_right(col_relative as usize)?;
            // println!("Moving right {}", col_relative);
        } else if col_relative < 0 {
            term.move_cursor_left((-col_relative) as usize)?;
            // println!("Moving left {}", -col_relative);
        }
        Ok(())
    }

    fn clear_line(&mut self, term: &mut Term) -> io::Result<()> {
        term.clear_line()?;
        let old_index = self.buf_index;
        let width = self.terminal_width;
        self.buf_index = old_index - old_index % width;
        Ok(())
    }

    fn write_all(&mut self, term: &mut Term, bytes: &[u8]) -> io::Result<()> {
        term.write_all(bytes)?;
        self.buf_index += bytes.len();
        Ok(())
    }

    fn write_line(&mut self, term: &mut Term, s: &str) -> io::Result<()> {
        term.write_line(s)?;
        self.buf_index += s.len();
        Ok(())
    }
}

/// Move the cursor to a target index. Passing terminal width explicitly because the terminal size may have changed
// fn move_cursor_given_index(
//     term: &mut Term, 
//     terminal_width: usize, 
//     current_index: usize, 
//     target_index: usize
// ) -> io::Result<()>
// {
//     let (current_line, current_row) = cursor_position(terminal_width, current_index);
//     let (target_line, target_row) = cursor_position(terminal_width, target_index);
//     // Move vertically
//     if target_line < current_line {
//         term.move_cursor_up(current_line - target_line)?;
//     } else if target_line > current_line {
//         term.move_cursor_down(target_line - current_line)?;
//     }
//     // Move horizontally
//     if target_row < current_row {
//         term.move_cursor_left(current_row - target_row)?;
//     } else if target_row > current_row {
//         term.move_cursor_right(target_row - current_row)?;
//     }
//     Ok(())
// }

fn write_char(term: &mut Term, buffer: &mut String, c: char, cursor: &mut Cursor) -> io::Result<()> {
    buffer.insert(cursor.buf_index, c);
    update_terminal(term, buffer, cursor)?;
    cursor.jump(term, 1);
    if cursor.buf_index % cursor.terminal_width == 0 {
        cursor.write_line(term, "")?;
    }
    Ok(())
}


fn delete_char(term: &mut Term, buffer: &mut String, cursor: &mut Cursor) -> io::Result<()> {
    // Remove a character. More cursor up if deleting past start of line 
    buffer.remove(cursor.buf_index-1);
    cursor.jump(term, -1)?;
    // term.write("Hello".as_bytes())?;
    // Special case when removing the last character in a line
    if  cursor.buf_index % cursor.terminal_width == 0 && cursor.buf_index == buffer.len() {
        term.clear_line()?;
        return Ok(())
    }
    update_terminal(term, buffer, cursor)?;
    Ok(())
}


fn update_terminal(term: &mut Term, buffer: &str, cursor: &mut Cursor) -> io::Result<()> {
    // Find the number of lines that need updating
    let old_cursor_index = cursor.buf_index;
    
    // Jump to the start of each line and rewrite going down
    cursor.jump(term, -(old_cursor_index as isize % cursor.terminal_width as isize))?;

    while cursor.buf_index != buffer.len() {
        cursor.clear_line(term)?;
        let i = cursor.buf_index;
        let j = (cursor.buf_index + cursor.terminal_width).min(buffer.len());
        cursor.write_all(term, &buffer.as_bytes()[i..j])?;
        if j != buffer.len() {
            cursor.write_line(term, "");
        }
    }

    // Jump back to previous position
    let jump_back_length =  old_cursor_index as isize - cursor.buf_index as isize;
    cursor.jump(term, jump_back_length);
    Ok(())
}
