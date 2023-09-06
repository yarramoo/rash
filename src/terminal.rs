use console::{self, Key, Term};
use std::{io::{self, Write}, arch::x86_64::_MM_FROUND_CUR_DIRECTION};

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
                if cursor.buf_index == buffer.len() { continue; }
                cursor.jump(&mut term, 1)?;
            },
            Key::ArrowLeft  => {
                if cursor.buf_index == PROMPT_STR.len() { continue; }
                cursor.jump(&mut term, -1)?;
            },
            Key::ArrowUp => {
                if cursor.buf_index < cursor.terminal_width { continue; }
                cursor.jump_checked(&mut term, -(terminal_width as isize), PROMPT_STR.len(), buffer.len())?;
            },
            Key::ArrowDown => {
                if cursor.buf_index >= cursor.terminal_width * (buffer.len() / cursor.terminal_width) { continue; }
                cursor.jump_checked(&mut term, terminal_width as isize, PROMPT_STR.len(), buffer.len())?;
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

    fn jump(&mut self, term: &mut Term, jump_length: isize) -> io::Result<()> {
        // Get current and target coordinates
        let (cur_line, cur_col) = self.cursor_position();
        self.update_index(jump_length);
        let (target_line, target_col) = self.cursor_position();
        // Move cursor in terminal
        self.move_relative(
            term,
            target_line as isize - cur_line as isize,
            target_col  as isize - cur_col as isize
        )?;
        Ok(())
    }

    fn move_relative(&mut self, term: &mut Term, line_relative: isize, col_relative: isize) -> io::Result<()> {
        use std::cmp::Ordering as O;
        // Move vertically
        match line_relative.cmp(&0) {
            O::Greater => term.move_cursor_down(line_relative as usize)?,
            O::Less    => term.move_cursor_up((-line_relative) as usize)?,
            O::Equal   => {}
        };
        // Move horizontally
        match col_relative.cmp(&0) {
            O::Greater => term.move_cursor_right(col_relative as usize)?,
            O::Less    => term.move_cursor_left((-col_relative) as usize)?,
            O::Equal   => {}
        };
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

fn write_char(term: &mut Term, buffer: &mut String, c: char, cursor: &mut Cursor) -> io::Result<()> {
    buffer.insert(cursor.buf_index, c);
    update_terminal(term, buffer, cursor)?;
    cursor.jump(term, 1)?;
    if cursor.buf_index % cursor.terminal_width == 0 {
        cursor.write_line(term, "")?;
    }
    Ok(())
}


fn delete_char(term: &mut Term, buffer: &mut String, cursor: &mut Cursor) -> io::Result<()> {
    // Remove a character. More cursor up if deleting past start of line 
    buffer.remove(cursor.buf_index-1);
    cursor.jump(term, -1)?;
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
            cursor.write_line(term, "")?;
        }
    }

    // Jump back to previous position
    let jump_back_length =  old_cursor_index as isize - cursor.buf_index as isize;
    cursor.jump(term, jump_back_length)?;
    Ok(())
}
