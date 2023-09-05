use std::fmt::Debug;
use std::process::{Command, Child, exit};
use std::error::Error;
use std::fmt;

use terminal::test_term_size;

mod terminal;

const PROMPT_STR: &'static str = &"rash>-";

fn main() {


    let cmd = terminal::get_cmd_interactive();
    if let Ok(cmd) = cmd {
        println!("Received: {cmd}");
        let mut cmd_vec = cmd.split(' ').collect::<Vec<_>>();
        let p = Program::from_args(cmd_vec).unwrap();
        let mut c = p.spawn().unwrap();
        c.wait();
    }
}

fn prompt() {
    print!("{}", PROMPT_STR);
}

struct Program<'a> {
    name: &'a str,
    arguments: Vec<&'a str>,
}

impl<'a> Program<'a> {
    fn new(name: &'a str, arguments: Vec<&'a str>) -> Self {
        Self { name, arguments }
    }
    
    fn from_args(mut arguments: Vec<&'a str>) -> Result<Self, &'static str> {
        if arguments.len() < 1 {
            return Err("Cannot build program from empty string");
        }
        Ok(Self { name: arguments.remove(0), arguments })
    }

    fn spawn(&self) -> Result<Child, String> {
        Command::new(self.name)
            .args(self.arguments.iter())
            .spawn()
            .or(Err(format!("Could not spawn from {:?}", &self)))
    }
}

impl<'a> Debug for Program<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgramArgs")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .finish()
    }
}

// A shell with specs similar to the Anubis shell from 3300. I would like it to have some extra features like autofilling using tab
// Full regex functionality at some point would be cool
// Some built ins like cd, exit, path

// Shell will prompt user for input
// Shell will accept input interactively
//  Shell can suggest auto completion. Auto complete of programs on the path, auto complete of files and directories too
// Shell will parse input into command structure
// Shell will execute in a non-blocking way
