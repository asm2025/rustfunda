use anyhow::{Result, anyhow};
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use dialoguer::{Select, theme::ColorfulTheme};
use rpassword::read_password;
use std::{
    io::{Write, stdin, stdout},
    time::Duration,
};

pub fn display_menu(items: &[&str], prompt: Option<&str>) -> Result<usize> {
    clear_screen()?;

    let prompt = match prompt {
        Some(s) if !s.is_empty() => s,
        _ => "Please select an option",
    };
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?;
    Ok(if selection == items.len() - 1 {
        0
    } else {
        selection + 1
    })
}

pub fn get(prompt: Option<&str>) -> Result<String> {
    print_prompt(prompt);

    let mut buffer = String::new();
    stdin()
        .read_line(&mut buffer)
        .expect("Failed to get input.");

    if !buffer.is_empty() {
        // Remove the trailing newlines
        buffer.pop();
    }

    Ok(buffer)
}

pub fn get_str(prompt: Option<&str>) -> Result<String> {
    let input = get(prompt)?;

    if input.is_empty() {
        return Err(anyhow!("No input provided"));
    }

    Ok(input)
}

pub fn get_char(prompt: Option<&str>) -> Result<char> {
    print_prompt(prompt);
    // Enable raw mode to read single characters
    enable_raw_mode()?;
    clear_keys();

    let result = loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Char(c) => break Ok(c),
                KeyCode::Esc | KeyCode::Enter => break Err(anyhow!("No input provided")),
                _ => continue,
            }
        }
    };

    // Disable raw mode before returning
    disable_raw_mode()?;
    println!();
    result
}

pub fn get_key(prompt: Option<&str>) -> Result<KeyEvent> {
    print_prompt(prompt);
    // Enable raw mode to read single characters
    enable_raw_mode()?;
    clear_keys();

    let result = loop {
        if let Ok(Event::Key(key_event)) = event::read() {
            break Ok(key_event);
        }
    };

    // Disable raw mode before returning
    disable_raw_mode()?;
    println!();
    result
}

pub fn get_numeric<T>(prompt: Option<&str>) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    let input = get_str(prompt)?;
    match input.parse::<T>() {
        Ok(number) => Ok(number),
        Err(e) => Err(anyhow!("{}", e)),
    }
}

pub fn get_password(prompt: Option<&str>) -> Result<String> {
    print_prompt(prompt);

    let input = read_password()?;
    Ok(input)
}

pub fn get_password_str(prompt: Option<&str>) -> Result<String> {
    let input = get_password(prompt)?;

    if input.is_empty() {
        return Err(anyhow!("No password provided"));
    }

    Ok(input)
}

pub fn pause() {
    print!("Press any key to continue...");
    get_key(None).unwrap();
}

pub fn clear_screen() -> Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    Ok(())
}

pub fn clear_keys() {
    while let Ok(_) = event::poll(Duration::from_secs(0)) {
        let _ = event::read();
    }
}

fn print_prompt(prompt: Option<&str>) {
    if let Some(p) = prompt {
        if !p.is_empty() {
            print!("{} ", p);
            stdout().flush().expect("Failed to flush stdout");
        }
    }
}
