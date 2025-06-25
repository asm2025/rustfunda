use anyhow::{Result, anyhow};
use crossterm::{
    ExecutableCommand, cursor,
    terminal::{Clear, ClearType},
};
use dialoguer::{Select, theme::ColorfulTheme};
use rpassword::read_password;
use std::io::{Write, stdin, stdout};

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

pub fn input(prompt: Option<&str>) -> Result<String> {
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

pub fn sinput(prompt: Option<&str>) -> Result<String> {
    let value = input(prompt)?;

    if value.is_empty() {
        return Err(anyhow!("No input provided"));
    }

    Ok(value)
}

pub fn ninput<T>(prompt: Option<&str>) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    let value = sinput(prompt)?;
    match value.parse::<T>() {
        Ok(number) => Ok(number),
        Err(e) => Err(anyhow!("{}", e)),
    }
}

pub fn password(prompt: Option<&str>) -> Result<String> {
    print_prompt(prompt);

    let value = read_password()?;
    Ok(value)
}

pub fn spassword(prompt: Option<&str>) -> Result<String> {
    let password = password(prompt)?;

    if password.is_empty() {
        return Err(anyhow!("No password provided"));
    }

    Ok(password)
}

pub fn pause() {
    println!("Press any key to continue...");
    input(None).unwrap();
}

pub fn clear_screen() -> Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    Ok(())
}

fn print_prompt(prompt: Option<&str>) {
    if let Some(p) = prompt {
        if !p.is_empty() {
            print!("{} ", p);
            stdout().flush().expect("Failed to flush stdout");
        }
    }
}
