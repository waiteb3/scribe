use std::error::Error;
use std::env::VarError;
use std::fmt;

#[derive(Debug)]
pub struct InitError {
    cause: String,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Error for InitError {}

impl std::convert::From<VarError> for InitError {
    fn from(err: VarError) -> Self {
        match err {
            VarError::NotPresent => {
                InitError{ cause: "SHELL env-var was not defined".to_owned() }
            }
            VarError::NotUnicode(text) => {
                InitError{ cause: format!("SHELL env-var encountered a unicode error: {:?}", text) }
            }
        }
    }
}

enum Shell {
    ZSH,
    FISH,
    BASH,
}

fn current_shell() -> Result<Shell, InitError> {
    let shell = std::env::var("SHELL")?.as_str().to_lowercase();
    if shell.contains("zsh") {
        Ok(Shell::ZSH)
    } else if shell.contains("fish") {
        Ok(Shell::FISH)
    } else if shell.contains("bash") {
        Ok(Shell::BASH)
    } else {
        Err(InitError{
            cause: format!("'{}' is not a supported shell", shell),
        })
    }
}

pub fn env_init() -> Result<(), InitError> {
    match current_shell()? {
        Shell::ZSH => {
            println!("{}", include_str!("etc/init.zsh"));
        }
        Shell::FISH => {
            println!("{}", include_str!("etc/init.fish"));
        }
        Shell::BASH => {
            println!("{}", include_str!("etc/init.bash"));
        }
    }
    Ok(())
}

pub fn import_history() -> Result<(), InitError>  {
    match current_shell()? {
        Shell::ZSH => {
        }
        Shell::FISH => {
        }
        Shell::BASH => {
        }
    }
    Ok(())
}