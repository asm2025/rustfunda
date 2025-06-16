use once_cell::sync::Lazy;
use std::{
    sync::{Arc, RwLock, atomic::AtomicBool},
    thread, time,
};
use util::io::input;

#[derive(Debug, Clone)]
struct Users {
    inner: Arc<RwLock<Vec<String>>>,
    changed: Arc<AtomicBool>,
}

impl Users {
    pub fn new() -> Self {
        Users {
            inner: Arc::new(RwLock::new(vec![])),
            changed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_users(&self) -> Vec<String> {
        let users = self.inner.read().unwrap();
        users.clone()
    }

    pub fn add_user(&self, name: String) {
        let mut users = self.inner.write().unwrap();
        users.push(name);
        self.changed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn has_changed(&self) -> bool {
        self.changed.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn reset_changed(&self) {
        self.changed
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

static USERS: Lazy<Users> = Lazy::new(Users::new);

fn main() {
    thread::spawn(|| {
        loop {
            let users = &*USERS;

            if !users.has_changed() {
                continue;
            }

            users.reset_changed();
            let users = users.get_users();
            println!("Current users: {:?}", users);
            thread::sleep(time::Duration::from_secs(3));
        }
    });

    loop {
        let name = input(Some("Enter a user name to add (or press ENTER to quit): ")).unwrap();
        if name.is_empty() {
            break;
        }
        USERS.add_user(name);
    }
}
