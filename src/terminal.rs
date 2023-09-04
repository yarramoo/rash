
// Interactive shell that allows users to input commands and tracks the command string as it is being typed
// Use this later to auto suggest based on current unfinished input
fn get_cmd_interactive() -> io::Result<String> {
    // Make a terminal stdout handle
    let mut term = Term::stdout();
    // Terminal height and width
    let (_t_height, t_width) = term.size();
    // String buffer to hold command
    let mut buf = String::new();
    // Indices into the buffer to track the bottom-most line being edited
    let (mut i, mut j) = (0, 0);
    // Loop on key input
    loop {
        let key = term.read_key()?;
        match key {
            Key::Char(c) => {
                // Add the character. Add a new line if the current terminal line is full.
                // Clear the line and reprint the line with the new character
                buf.push(c);
                if j - i == t_width as usize - 1 {
                    term.write_line("")?;
                    i = j;
                }
                term.clear_line()?;
                j += 1;
                term.write_all(&buf.as_bytes()[i..j])?;
            },
            Key::Backspace => {
                // Delete a character. Reduce the slice of the buffer that is shown.
                // If the current terminal line empties then move the cursor up and to the right to edit the above line
                // Adjust slice into buffer
                let deleted_char = buf.pop();
                if deleted_char.is_none() { continue; }
                j -= 1;
                term.clear_line()?;
                if i == j && i != 0 {
                    term.move_cursor_up(1)?;
                    term.move_cursor_right(t_width as usize)?;
                    i = j - (t_width-1) as usize;
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
            }
            _ => {},
        };
    }
    Ok(buf)
}