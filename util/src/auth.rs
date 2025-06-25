use anyhow::{Result, anyhow};
use fake::{
    Dummy,
    faker::{
        internet::en::{Password as FakePassword, SafeEmail},
        name::en::Name,
    },
};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize, Dummy)]
pub enum UserRole {
    #[default]
    None,
    User,
    Admin,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::None => write!(f, "-"),
            UserRole::User => write!(f, "User"),
            UserRole::Admin => write!(f, "Admin"),
        }
    }
}

impl From<String> for UserRole {
    fn from(role: String) -> Self {
        match role.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "user" => UserRole::User,
            _ => UserRole::None,
        }
    }
}

impl From<&str> for UserRole {
    fn from(role: &str) -> Self {
        String::from(role).into()
    }
}

impl From<i32> for UserRole {
    fn from(role: i32) -> Self {
        match role {
            1 => UserRole::User,
            2 => UserRole::Admin,
            _ => UserRole::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct User {
    id: Uuid,
    #[dummy(faker = "SafeEmail()")]
    username: String,
    #[dummy(faker = "FakePassword(8..16)")]
    password: String,
    #[dummy(faker = "Name()")]
    name: String,
    role: UserRole,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.id, self.username)
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            username: String::new(),
            password: String::new(),
            name: String::new(),
            role: UserRole::None,
        }
    }
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build() -> Self {
        Self::new()
    }

    pub fn with(
        mut self,
        id: &Uuid,
        name: &str,
        username: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Self {
        self.id = id.to_owned();
        self.username = username.to_string();
        self.password = password_hash.to_string();
        self.name = name.to_string();
        self.role = role;
        self
    }

    pub fn with_id(mut self, id: &Uuid) -> Self {
        self.id = id.to_owned();
        self
    }

    pub fn with_username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }

    pub fn with_password(mut self, password_hash: &str) -> Self {
        self.password = password_hash.to_string();
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn set_id(&mut self, value: &Uuid) {
        self.id = value.to_owned();
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn set_username(&mut self, value: &str) {
        self.username = value.to_string();
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn set_password(&mut self, value: &str) {
        self.password = value.to_string();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: &str) {
        self.name = value.to_string();
    }

    pub fn role(&self) -> UserRole {
        self.role
    }

    pub fn set_role(&mut self, value: UserRole) {
        self.role = value;
    }

    pub fn is_valid_for_update(&self) -> bool {
        !self.id.is_nil() && !self.username.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid_for_update() && !self.password.is_empty() && self.role != UserRole::None
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn is_user(&self) -> bool {
        self.role == UserRole::User
    }
}

#[derive(Debug)]
pub struct Column {
    name: String,
    width: usize,
    property: String,
}

impl Column {
    const WIDTH_MIN: usize = 4;
    const WIDTH_DEF: usize = 10;

    pub fn new(name: &str, width: usize, property: &str) -> Self {
        Self {
            name: name.to_string(),
            width: if width > Self::WIDTH_MIN {
                width
            } else if width > 0 {
                Self::WIDTH_MIN
            } else {
                Self::WIDTH_DEF
            },
            property: property.trim().to_lowercase(),
        }
    }
}

pub struct UserFormatter {
    columns: Vec<Column>,
}

impl Default for UserFormatter {
    fn default() -> Self {
        Self {
            columns: vec![
                Column::new("ID", 36, "id"),
                Column::new("Username", 20, "username"),
                Column::new("Role", 10, "role"),
            ],
        }
    }
}

impl UserFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_columns(columns: Vec<Column>) -> Result<Self> {
        if columns.is_empty() {
            return Err(anyhow!("Columns cannot be empty"));
        }

        Ok(Self { columns })
    }

    pub fn print_headers(&self) {
        let mut line = String::new();

        for column in &self.columns {
            line.push_str(&format!("{:<width$} ", column.name, width = column.width));
        }

        println!("{}", line);
        self.print_separator();
    }

    pub fn print_user(&self, user: &User) {
        let mut line = String::new();

        for column in &self.columns {
            let value = match column.property.as_str() {
                "id" => user.id().to_string(),
                "username" => user.username().to_string(),
                "password" => user.password().to_string(),
                "name" => user.name().to_string(),
                "role" => user.role().to_string(),
                _ => String::from(""),
            };

            // Truncate if value is longer than column width
            let formatted_value = if value.len() > column.width {
                format!("{}...", &value[0..column.width - 3])
            } else {
                value
            };

            line.push_str(&format!(
                "{:<width$} ",
                formatted_value,
                width = column.width
            ));
        }

        println!("{}", line);
    }

    pub fn print_users(&self, users: &[User]) {
        if users.is_empty() {
            println!("No users found.");
            return;
        }

        self.print_headers();

        for user in users {
            self.print_user(&user);
        }

        self.print_separator();
        println!("Total users: {}", users.len());
    }

    pub fn print_separator(&self) {
        let line =
            "-".repeat(self.columns.iter().map(|c| c.width).sum::<usize>() + self.columns.len());
        println!("{}", line);
    }
}
