use clap::{CommandFactory, Parser, Subcommand};
use crossterm::{
    ExecutableCommand, cursor,
    terminal::{Clear, ClearType},
};
use std::{io::stdout, path::Path};
use uuid::Uuid;

use authentication::*;
use util::{
    Result,
    auth::{User, UserFormatter, UserRole},
    io::pause,
};

#[derive(Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to the system
    Login {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
    /// List all users
    List,
    /// List users by role
    ListByRole {
        #[arg(short, long)]
        role: UserRole,
    },
    /// Add a new user
    Add {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        role: UserRole,
    },
    /// Update an existing user
    Update {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        new_name: Option<String>,
        #[arg(short, long)]
        new_username: Option<String>,
        #[arg(short, long)]
        new_password: Option<String>,
        #[arg(short, long)]
        new_role: Option<UserRole>,
    },
    /// Remove a user
    Remove {
        #[arg(short, long)]
        username: String,
    },
}

fn main() {
    clear_screen().unwrap();
    println!("Welcome to the Login System!");

    let cli = Args::parse();
    let mut user_store =
        UserStore::load_from_file(Path::new("../users.json")).unwrap_or_else(|ex| {
            eprintln!("{}", ex);
            std::process::exit(1);
        });
    match cli.command {
        Some(Commands::Login { username, password }) => {
            if let Err(ex) = login(&user_store, &username, &password) {
                eprintln!("{}", ex);
            }
        }
        Some(Commands::List) => {
            if let Err(ex) = list_users(&user_store) {
                eprintln!("{}", ex);
            }
        }
        Some(Commands::ListByRole { role }) => {
            if let Err(ex) = list_users_by_role(&user_store, role) {
                eprintln!("{}", ex);
            }
        }
        Some(Commands::Add {
            name,
            username,
            password,
            role,
        }) => {
            if let Err(ex) = add_user(&mut user_store, &name, &username, &password, role) {
                eprintln!("{}", ex);
            }
        }
        Some(Commands::Update {
            username,
            new_name,
            new_username,
            new_password,
            new_role,
        }) => {
            if let Err(ex) = update_user(
                &mut user_store,
                &username,
                new_name.as_deref(),
                new_username.as_deref(),
                new_password.as_deref(),
                new_role.unwrap_or(UserRole::None),
            ) {
                eprintln!("{}", ex);
            }
        }
        Some(Commands::Remove { username }) => {
            if let Err(ex) = remove_user(&mut user_store, &username) {
                eprintln!("{}", ex);
            }
        }
        None => {
            let mut cmd = Args::command();
            cmd.print_help().unwrap_or_else(|e| {
                eprintln!("Error displaying help: {}", e);
                std::process::exit(1);
            });
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

fn login(user_store: &UserStore, username: &str, password: &str) -> Result<()> {
    if let Ok(user) = user_store.login(&username, &password) {
        println!("{}", user_store.great_user(&user.username()));
        match user.role() {
            UserRole::Admin => println!("You are logged in as an Admin."),
            UserRole::User => println!("You are logged in as a User."),
            UserRole::None => println!("You are logged in with no role."),
        }
        pause();
    } else {
        return Err("Invalid credentials. Please try again.".into());
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

fn list_users_by_role(user_store: &UserStore, role: UserRole) -> Result<()> {
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

fn add_user(
    user_store: &mut UserStore,
    name: &str,
    username: &str,
    password: &str,
    role: UserRole,
) -> Result<()> {
    clear_screen()?;

    let user = User::build().with(
        &Uuid::new_v4(),
        name,
        username,
        &user_store.hash_password(password),
        role,
    );
    user_store.add(user)?;
    user_store.save_to_file(Path::new("../users.json"))?;
    println!("User '{}' added successfully.", username);
    pause();
    Ok(())
}

fn update_user(
    user_store: &mut UserStore,
    username: &str,
    new_name: Option<&str>,
    new_username: Option<&str>,
    new_password: Option<&str>,
    nw_role: UserRole,
) -> Result<()> {
    let mut user = user_store
        .get_by_username(&username)
        .cloned()
        .ok_or_else(|| format!("User '{}' not found.", username))?;

    if let Some(new_name) = new_name {
        user.set_name(new_name);
    }

    if let Some(new_username) = new_username {
        user.set_username(new_username);
    }

    if let Some(new_password) = new_password {
        user.set_password(&user_store.hash_password(new_password));
    }

    if nw_role != UserRole::None {
        user.set_role(nw_role);
    }

    user_store.update(user)?;
    user_store.save_to_file(Path::new("../users.json"))?;
    println!("User '{}' updated successfully.", username);
    pause();
    Ok(())
}

fn remove_user(user_store: &mut UserStore, username: &str) -> Result<()> {
    if user_store.remove_by_username(&username).is_ok() {
        user_store.save_to_file(Path::new("../users.json"))?;
        println!("User '{}' removed successfully.", username);
    } else {
        println!("User '{}' not found.", username);
    }

    pause();
    Ok(())
}
