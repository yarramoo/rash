use console::{self, Term};
use console::Key;
use std::io::{self, Write};
use std::process::{Command, Child};
use std::error::Error;

mod terminal;

fn main() {
    let cmd = get_cmd_interactive();
    if let Ok(cmd) = cmd {
        let mut cmd_vec = cmd.split(' ').collect::<Vec<_>>();
        let p = Program::from_args(cmd_vec).unwrap();
        let mut c = p.spawn().unwrap();
        c.wait();
    }
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

    fn spawn(&self) -> Result<Child, &str> {
        Command::new(self.name)
            .args(self.arguments.iter())
            .spawn()
            .or(Err("Could not spawn child from this program"))
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
