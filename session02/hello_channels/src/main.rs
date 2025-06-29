use serde::{Deserialize, Serialize};
use std::{fmt, sync::mpsc};
use util::{io::get, threading::Signal};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
enum Command {
    #[default]
    None,
    Hello(String),
    Say(String),
    Quit,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::None => write!(f, ""),
            Command::Hello(name) => write!(f, "Hello, {}", name),
            Command::Say(message) => write!(f, "{}", message),
            Command::Quit => write!(f, "Bye"),
        }
    }
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        if s.is_empty() {
            return Command::None;
        }

        let l = s.to_lowercase();
        match l.as_str() {
            _ if l.starts_with("hello ") => Command::Hello(s[6..].to_string()),
            _ if l.starts_with("say ") => Command::Say(s[4..].to_string()),
            "quit" => Command::Quit,
            _ => Command::None,
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let signal = Signal::new();
    let signal2 = signal.clone();
    let handle = std::thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            println!("Bot: {}", command);
            signal2.set();
            if command == Command::Quit {
                break;
            }
        }
    });

    loop {
        let input = get(Some(">")).unwrap();
        let command = Command::from(input);

        if let Err(ex) = tx.send(command.clone()) {
            eprintln!("{}", ex);
            break;
        }

        if command == Command::Quit {
            break;
        }

        signal.wait();
    }

    if let Err(ex) = handle.join() {
        eprintln!("Error joining thread: {:?}", ex);
    }
}
