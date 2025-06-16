use bimap::BiMap;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use util::{
    Result,
    auth::{User, UserRole},
};
use uuid::Uuid;

pub struct UserStore {
    users: HashMap<Uuid, User>,
    username_map: BiMap<String, Uuid>,
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

    pub fn from(users: HashMap<Uuid, User>) -> Self {
        let mut username_map = BiMap::new();

        for user in users.values() {
            username_map.insert(user.username().to_owned(), user.id().clone());
        }

        Self {
            users,
            username_map,
        }
    }

    pub fn load_from_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        let path = path.as_ref();
        let users: HashMap<Uuid, User> = {
            if !path.exists() {
                let mut map: HashMap<Uuid, User> = HashMap::new();
                add_default_users(&mut map);
                let json = serde_json::to_string(&map)?;
                std::fs::write(path, json).expect("Unable to write users file");
                map
            } else {
                let data = std::fs::read_to_string(path)?;
                let mut map: HashMap<Uuid, User> =
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

        if self.users.contains_key(user.id()) || self.username_map.contains_left(user.username()) {
            return Err("User already exists".into());
        }

        self.users.insert(user.id().clone(), user.clone());
        self.username_map
            .insert(user.username().to_owned(), user.id().clone());
        Ok(())
    }

    pub fn update(&mut self, user: User) -> Result<()> {
        if !user.is_valid_for_update() {
            return Err("Invalid user data".into());
        }

        if let Some(existing_user) = self.users.get(user.id()) {
            if existing_user.username() != user.username()
                && self.username_map.contains_left(user.username())
            {
                return Err("Username already exists".into());
            }

            let mut user = user;

            if user.password().is_empty() {
                // If the password is empty, keep the existing password hash
                user.set_password(existing_user.password());
            }

            if user.role() == UserRole::None {
                // If the role is None, keep the existing role
                user.set_role(existing_user.role());
            }

            // Update the username map only if the username has changed
            if existing_user.username() != user.username() {
                self.username_map.remove_by_left(existing_user.username());
                self.username_map
                    .insert(user.username().to_owned(), user.id().clone());
            }

            self.users.insert(user.id().clone(), user.clone());
            self.username_map
                .insert(user.username().to_owned(), user.id().clone());
        } else {
            self.users.insert(user.id().clone(), user.clone());
            self.username_map
                .insert(user.username().to_owned(), user.id().clone());
        }

        Ok(())
    }

    pub fn remove(&mut self, id: &Uuid) -> Result<()> {
        if let Some(user) = self.users.remove(id) {
            self.username_map.remove_by_right(user.id());
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
            .filter(|user| user.role() == role)
            .cloned()
            .collect()
    }

    pub fn get(&self, id: &Uuid) -> Option<&User> {
        if id.is_nil() {
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

        if self.verify_password(password, user.password()) {
            Ok(user.clone())
        } else {
            Err("Invalid credentials".into())
        }
    }

    pub fn great_user(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}

fn add_default_users(users: &mut HashMap<Uuid, User>) {
    let usernames = users
        .values()
        .map(|u| u.username().to_owned())
        .collect::<HashSet<String>>();

    if !usernames.contains("admin") {
        let user = User::build().with(
            &Uuid::new_v4(),
            "administrator",
            "admin",
            &hash_password("root"),
            UserRole::Admin,
        );
        users.insert(user.id().clone(), user);
    }

    if !usernames.contains("user") {
        let user = User::build().with(
            &Uuid::new_v4(),
            "User",
            "user",
            &hash_password("password"),
            UserRole::User,
        );
        users.insert(user.id().clone(), user);
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
