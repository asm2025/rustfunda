use rpassword::read_password;
use std::io::{Write, stdin, stdout};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn input(prompt: Option<&str>) -> Result<String> {
    print_prompt(prompt);

    let mut buffer = String::new();
    stdin()
        .read_line(&mut buffer)
        .expect("Failed to get input.");
    let buffer = buffer.trim();
    Ok(buffer.to_string())
}

pub fn sinput(prompt: Option<&str>) -> Result<String> {
    let value = input(prompt)?;

    if value.is_empty() {
        return Err("No input provided".into());
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
        Err(e) => Err(Box::new(e)),
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
        return Err("No password provided".into());
    }

    Ok(password)
}

pub fn pause() {
    println!("Press any key to continue...");
    input(None).unwrap();
}

fn print_prompt(prompt: Option<&str>) {
    let prompt = prompt
        .and_then(|p| if p.is_empty() { None } else { Some(p) })
        .map(String::from);
    match prompt {
        Some(p) => {
            print!("{}", p);
            stdout().flush().expect("Failed to flush stdout");
        }
        None => {}
    }
}
