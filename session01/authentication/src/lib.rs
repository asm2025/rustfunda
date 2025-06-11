use bimap::BiMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::Path,
};
use util::Result;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    #[default]
    None,
    User,
    Admin,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::None => write!(f, "None"),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: String,
    username: String,
    password_hash: String,
    role: UserRole,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: String::new(),
            username: String::new(),
            password_hash: String::new(),
            role: UserRole::None,
        }
    }
}

impl User {
    pub fn new(username: &str, password_hash: &str, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            role,
        }
    }

    pub fn build() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn with_name(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }

    pub fn with_password(mut self, password_hash: &str) -> Self {
        self.password_hash = password_hash.to_string();
        self
    }

    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_id(&mut self, value: &str) {
        self.id = value.to_string();
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn set_username(&mut self, value: &str) {
        self.username = value.to_string();
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn set_password_hash(&mut self, value: &str) {
        self.password_hash = value.to_string();
    }

    pub fn role(&self) -> UserRole {
        self.role
    }

    pub fn set_role(&mut self, value: UserRole) {
        self.role = value;
    }

    pub fn is_valid_for_update(&self) -> bool {
        !self.id.is_empty() && !self.username.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid_for_update() && !self.password_hash.is_empty() && self.role != UserRole::None
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
            return Err("Columns cannot be empty".into());
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

pub struct UserStore {
    users: HashMap<String, User>,
    username_map: BiMap<String, String>,
}

impl UserStore {
    pub fn new() -> Self {
        let users = HashMap::new();
        let username_map = BiMap::new();
        Self {
            users,
            username_map,
        }
    }

    pub fn from(users: HashMap<String, User>) -> Self {
        let mut username_map = BiMap::new();

        for user in users.values() {
            username_map.insert(user.username.clone(), user.id.clone());
        }

        Self {
            users,
            username_map,
        }
    }

    pub fn load_from_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        let path = path.as_ref();
        let users: HashMap<String, User> = {
            if !path.exists() {
                let mut map: HashMap<String, User> = HashMap::new();
                add_default_users(&mut map);
                let json = serde_json::to_string(&map)?;
                std::fs::write(path, json).expect("Unable to write users file");
                map
            } else {
                let data = std::fs::read_to_string(path)?;
                let mut map: HashMap<String, User> =
                    serde_json::from_str(&data).map_err(|e| e.to_string())?;
                map.retain(|_, user| user.is_valid());
                add_default_users(&mut map);
                map
            }
        };
        Ok(Self::from(users))
    }

    pub fn save_to_file<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string(&self.users)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn hash_password(&self, password: &str) -> String {
        hash_password(password)
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> bool {
        verify_password(password, password_hash)
    }

    pub fn add(&mut self, user: User) -> Result<()> {
        if !user.is_valid() {
            return Err("Invalid user data".into());
        }

        if self.users.contains_key(&user.id) || self.username_map.contains_left(&user.username) {
            return Err("User already exists".into());
        }

        self.users.insert(user.id.clone(), user.clone());
        self.username_map
            .insert(user.username.clone(), user.id.clone());
        Ok(())
    }

    pub fn update(&mut self, user: User) -> Result<()> {
        if !user.is_valid_for_update() {
            return Err("Invalid user data".into());
        }

        if let Some(existing_user) = self.users.get(&user.id) {
            if existing_user.username != user.username
                && self.username_map.contains_left(&user.username)
            {
                return Err("Username already exists".into());
            }

            let mut user = user;

            if user.password_hash.is_empty() {
                // If the password is empty, keep the existing password hash
                user.password_hash = existing_user.password_hash.clone();
            }

            if user.role == UserRole::None {
                // If the role is None, keep the existing role
                user.role = existing_user.role;
            }

            // Update the username map only if the username has changed
            if existing_user.username != user.username {
                self.username_map.remove_by_left(&existing_user.username);
                self.username_map
                    .insert(user.username.clone(), user.id.clone());
            }

            self.users.insert(user.id.clone(), user.clone());
            self.username_map
                .insert(user.username.clone(), user.id.clone());
        } else {
            self.users.insert(user.id.clone(), user.clone());
            self.username_map
                .insert(user.username.clone(), user.id.clone());
        }

        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Result<()> {
        if let Some(user) = self.users.remove(id) {
            self.username_map.remove_by_right(&user.id);
            Ok(())
        } else {
            Err("User not found".into())
        }
    }

    pub fn remove_by_username(&mut self, username: &str) -> Result<()> {
        match self.username_map.get_by_left(username) {
            Some(id) => self.remove(&id.clone()),
            None => Err("User not found".into()),
        }
    }

    pub fn clear(&mut self) {
        self.users.clear();
        self.username_map.clear();
    }

    pub fn users(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }

    pub fn users_by_role(&self, role: UserRole) -> Vec<User> {
        self.users
            .values()
            .filter(|user| user.role == role)
            .cloned()
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&User> {
        if id.is_empty() {
            return None;
        }

        self.users.get(id)
    }

    pub fn get_by_username(&self, username: &str) -> Option<&User> {
        if username.is_empty() {
            return None;
        }

        self.username_map
            .get_by_left(username)
            .and_then(|id| self.users.get(id))
    }

    pub fn login(&self, username: &str, password: &str) -> Result<User> {
        if username.is_empty() || password.is_empty() {
            return Err("Username or password cannot be empty".into());
        }

        let username = username.trim().to_lowercase();
        let user = self
            .get_by_username(&username)
            .ok_or_else(|| "User not found".to_string())?;

        if self.verify_password(password, &user.password_hash) {
            Ok(user.clone())
        } else {
            Err("Invalid credentials".into())
        }
    }

    pub fn great_user(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}

fn add_default_users(users: &mut HashMap<String, User>) {
    let usernames = users
        .values()
        .map(|u| u.username.clone())
        .collect::<HashSet<String>>();

    if !usernames.contains("admin") {
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash: hash_password("root"),
            role: UserRole::Admin,
        };
        users.insert(user.id.clone(), user);
    }

    if !usernames.contains("user") {
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: "user".to_string(),
            password_hash: hash_password("password"),
            role: UserRole::User,
        };
        users.insert(user.id.clone(), user);
    }
}

pub fn hash_password(password: &str) -> String {
    if password.is_empty() {
        return String::new();
    }

    bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap_or_default()
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    if password.is_empty() || password_hash.is_empty() {
        return false;
    }

    bcrypt::verify(password, password_hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::User,
        };
        assert!(user.is_valid());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.role, UserRole::User);
    }

    #[test]
    fn test_user_store_add() {
        let mut store = UserStore::new();
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            password_hash: hash_password("hashed_password"),
            role: UserRole::User,
        };
        assert!(store.add(user).is_ok());
        assert!(store.get_by_username("testuser").is_some());
    }

    #[test]
    fn test_user_store_update() {
        let mut store = UserStore::new();
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::User,
        };
        store.add(user).unwrap();

        let updated_user = User {
            id: "1".to_string(),
            username: "updateduser".to_string(),
            password_hash: hash_password("new_hashed_password"),
            role: UserRole::User,
        };
        assert!(store.update(updated_user).is_ok());
        assert!(store.get_by_username("updateduser").is_some());
    }

    #[test]
    fn test_user_store_remove() {
        let mut store = UserStore::new();
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            password_hash: hash_password("hashed_password"),
            role: UserRole::User,
        };
        store.add(user).unwrap();
        assert!(store.remove("1").is_ok());
        assert!(store.get_by_username("testuser").is_none());
    }

    #[test]
    fn test_user_store_login() {
        let mut store = UserStore::new();
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            password_hash: hash_password("password"),
            role: UserRole::User,
        };
        store.add(user).unwrap();

        let login_result = store.login("testuser", "password");
        assert!(login_result.is_ok());
        assert_eq!(login_result.unwrap().username, "testuser");

        let failed_login = store.login("testuser", "wrongpassword");
        assert!(failed_login.is_err());
    }

    #[test]
    fn test_user_store_get_by_role() {
        let mut store = UserStore::new();
        let admin_user = User {
            id: "1".to_string(),
            username: "admin".to_string(),
            password_hash: hash_password("root"),
            role: UserRole::Admin,
        };
        let regular_user = User {
            id: "2".to_string(),
            username: "user".to_string(),
            password_hash: hash_password("password"),
            role: UserRole::User,
        };
        store.add(admin_user).unwrap();
        store.add(regular_user).unwrap();

        let admins = store.users_by_role(UserRole::Admin);
        assert_eq!(admins.len(), 1);
        assert_eq!(admins[0].username, "admin");

        let users = store.users_by_role(UserRole::User);
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "user");
    }

    #[test]
    fn test_user_store_hash_password() {
        let password = "password";
        let hashed = hash_password(password);
        assert!(!hashed.is_empty());
        assert!(verify_password(password, &hashed));
        assert!(!verify_password("wrongpassword", &hashed));
    }

    #[test]
    fn test_user_store_load_from_file() {
        let store = UserStore::load_from_file(Path::new("test_users.json"));
        assert!(store.is_ok());
        let store = store.unwrap();
        assert!(!store.users().is_empty());
        assert!(store.get_by_username("admin").is_some());
        assert!(store.get_by_username("user").is_some());
    }

    #[test]
    fn test_user_store_save_to_file() {
        let mut store = UserStore::new();
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::User,
        };
        store.add(user).unwrap();
        assert!(store.save_to_file(Path::new("test_users.json")).is_ok());
        let loaded_store = UserStore::load_from_file(Path::new("test_users.json")).unwrap();
        assert!(loaded_store.get_by_username("testuser").is_some());
    }

    #[test]
    fn test_user_formatter() {
        let user = User::new("testuser", "password", UserRole::Admin).with_id("12345");
        let formatter = UserFormatter::new();

        formatter.print_headers();
        formatter.print_user(&user);
    }

    #[test]
    fn test_custom_formatter() {
        let columns = vec![
            Column::new("Username", 15, "username"),
            Column::new("Role", 8, "role"),
        ];
        let formatter = UserFormatter::with_columns(columns).unwrap();
        let user = User::new("testuser", "password", UserRole::Admin);

        formatter.print_headers();
        formatter.print_user(&user);
    }
}
