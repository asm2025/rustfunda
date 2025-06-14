use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::thread;

static SHARED: Lazy<DashMap<u32, u32>> = Lazy::new(DashMap::new);

fn main() {
    for n in 0..100 {
        thread::spawn(move || {
            loop {
                if let Some(mut entry) = SHARED.get_mut(&n) {
                    *entry += 1;
                } else {
                    SHARED.insert(n, 1);
                }
            }
        });
    }

    thread::sleep(std::time::Duration::from_secs(2));
    println!("{SHARED:#?}");
}
