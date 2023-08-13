use std::fmt;
use std::io;
use std::process::{Command, ExitStatus, Output};

macro_rules! println_shell {
    ($($arg:tt)*) => ({
        println!("\x1b[90m$ {}\x1b[0m", format_args!($($arg)*));
    })
}

macro_rules! new {
    ($bin:expr, $($x:expr),* $(,)?) => {
        {
            let mut command = std::process::Command::new($bin);
            $(command.arg($x);)*
            crate::shell::Shell::new(command)
        }
    };
}

pub(crate) use new;

#[derive(Debug)]
pub struct Shell(Command);

impl Shell {
    pub fn new(command: Command) -> Self {
        Shell(command)
    }

    pub fn output(&mut self, print: bool) -> Result<Output, ShellError> {
        if print {
            println_shell!("{}\n", self);
        }

        self.0.output().map_err(|e| ShellError::Io {
            shell: self.to_string(),
            source: e,
        })
    }

    pub fn run(&mut self, print: bool) -> Result<(), ShellError> {
        if print {
            println_shell!("{}\n", self);
        }

        self.status()?;

        Ok(())
    }

    pub fn status(&mut self) -> Result<ExitStatus, ShellError> {
        self.0.status().map_err(|e| ShellError::Io {
            shell: self.to_string(),
            source: e,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShellError {
    #[error("error running \"{shell}\"")]
    Io {
        shell: String,
        #[source]
        source: io::Error,
    },
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.get_program().to_str().unwrap())?;

        for arg in self.0.get_args() {
            write!(f, " {}", arg.to_str().unwrap())?;
        }

        Ok(())
    }
}
