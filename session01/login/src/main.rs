use crossterm::{
    ExecutableCommand, cursor,
    terminal::{Clear, ClearType},
};
use dialoguer::{Select, theme::ColorfulTheme};
use std::{io::stdout, path::Path};

use authentication::*;
use util::*;

fn main() {
    clear_screen().unwrap();

    let mut user_store =
        UserStore::load_from_file(Path::new("../users.json")).unwrap_or_else(|ex| {
            eprintln!("{}", ex);
            std::process::exit(1);
        });

    loop {
        let choice: usize = display_menu().unwrap_or_else(|ex| {
            eprintln!("{}", ex);
            10
        });

        let result = match choice {
            1 => login(&user_store),
            2 => list_users(&user_store),
            3 => list_users_by_role(&user_store),
            4 => add_user(&mut user_store),
            5 => update_user(&mut user_store),
            6 => remove_user(&mut user_store),
            7 => save_users(&user_store),
            _ => {
                if choice == 0 {
                    println!("Exiting the application.");
                    std::process::exit(0);
                }

                Err("Invalid option. Please try again.".into())
            }
        };

        if let Err(ex) = result {
            eprintln!("{}", ex);
            pause();
        }
    }
}

fn clear_screen() -> Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    Ok(())
}

fn display_menu() -> Result<usize> {
    let items = vec![
        "Login",
        "List users",
        "List users by role",
        "Add user",
        "Update user",
        "Remove user",
        "Save users",
        "Exit",
    ];
    clear_screen()?;

    println!("Welcome to the Login System!");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select an option")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(if selection == items.len() - 1 {
        0
    } else {
        selection + 1
    })
}

fn login(user_store: &UserStore) -> Result<()> {
    let mut tries = 0;

    loop {
        let username = sinput(Some("Enter your username: "))?;
        let password = password(Some("Enter your password: "))?;

        if let Ok(user) = user_store.login(&username, &password) {
            println!("{}", user_store.great_user(&user.username()));
            match user.role() {
                authentication::UserRole::Admin => println!("You are logged in as an Admin."),
                authentication::UserRole::User => println!("You are logged in as a User."),
                authentication::UserRole::None => println!("You are logged in with no role."),
            }
            pause();
            break;
        }

        tries += 1;

        if tries >= 3 {
            eprintln!("Too many failed attempts.");
            pause();
            break;
        }

        eprintln!("Invalid credentials. Please try again.");
    }

    Ok(())
}

fn list_users(user_store: &UserStore) -> Result<()> {
    let users = user_store.users();

    if users.is_empty() {
        eprintln!("No users found.");
        pause();
        return Ok(());
    }

    clear_screen()?;

    let formatter = UserFormatter::default();
    formatter.print_users(&users);
    pause();
    Ok(())
}

fn list_users_by_role(user_store: &UserStore) -> Result<()> {
    let role: UserRole = sinput(Some("Enter role: "))
        .unwrap_or("none".to_string())
        .into();
    let users = user_store.users_by_role(role);

    if users.is_empty() {
        eprintln!("No users found with role '{}'.", role);
        pause();
        return Ok(());
    }

    clear_screen()?;

    let formatter = UserFormatter::default();
    formatter.print_users(&users);
    pause();
    Ok(())
}

fn add_user(user_store: &mut UserStore) -> Result<()> {
    let username = sinput(Some("Enter username: "))?;
    let password = spassword(Some("Enter password: "))?;
    let role: UserRole = sinput(Some("Enter role (leave empty for default): "))
        .unwrap_or("user".to_string())
        .into();
    let user = User::new(&username, &user_store.hash_password(&password), role);
    user_store.add(user)?;
    println!("User '{}' added successfully.", username);
    pause();
    Ok(())
}

fn update_user(user_store: &mut UserStore) -> Result<()> {
    let username = sinput(Some("Enter username to update: "))?;
    let mut user = user_store
        .get_by_username(&username)
        .cloned()
        .ok_or_else(|| format!("User '{}' not found.", username))?;
    let password = password(Some("Enter new password (leave empty to keep current): "))?;
    let role: UserRole = sinput(Some("Enter new role (leave empty to keep current): "))
        .unwrap_or("none".to_string())
        .into();
    if password.is_empty() && role == UserRole::None {
        println!("No changes made to user '{}'.", username);
        pause();
        return Ok(());
    }

    if !password.is_empty() {
        user.set_password_hash(&user_store.hash_password(&password));
    }

    user.set_role(role);
    user_store.update(user)?;
    println!("User '{}' updated successfully.", username);

    pause();
    Ok(())
}

fn remove_user(user_store: &mut UserStore) -> Result<()> {
    let username = sinput(Some("Enter username to remove: "))?;

    if user_store.remove_by_username(&username).is_ok() {
        println!("User '{}' removed successfully.", username);
    } else {
        eprintln!("User '{}' not found.", username);
    }

    pause();
    Ok(())
}

fn save_users(user_store: &UserStore) -> Result<()> {
    clear_screen()?;

    if user_store.save_to_file(Path::new("../users.json")).is_ok() {
        println!("Users saved successfully.");
    } else {
        eprintln!("Failed to save users.");
    }

    pause();
    Ok(())
}
