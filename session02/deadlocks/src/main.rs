use std::{mem, sync::Mutex, thread, time::Duration};

static SHARED: Mutex<u32> = Mutex::new(0);

fn poinsoner() {
    let mut shared = SHARED.lock().unwrap();
    *shared += 1;
    panic!("This thread will panic and not release the lock");
}

fn main() {
    let lock = SHARED.lock().unwrap();
    mem::drop(lock); // Explicitly drop the lock to release it

    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            let mut tries = 0;

            while tries < 10 {
                match SHARED.try_lock() {
                    Ok(mut shared) => {
                        *shared += 1;
                        println!(">>> {i} lock acquired");
                        break;
                    }
                    Err(ex) => {
                        match ex {
                            std::sync::TryLockError::Poisoned(_) => {
                                println!(">>> {i} lock is poisoned.");
                                return;
                            }
                            _ => {
                                // Sleep to avoid busy waiting
                                thread::sleep(Duration::from_millis(100));
                                tries += 1;
                            }
                        }
                    }
                }
            }
        });
        handles.push(handle);
    }

    let poison_handle = thread::Builder::new()
        .name("poisoner".to_string())
        .spawn(poinsoner)
        .unwrap();
    handles.push(poison_handle);

    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("Thread panicked: {:?}", e);
        }
    }

    if let Ok(shared) = SHARED.try_lock() {
        println!("Final value of shared: {}", *shared);
    } else {
        println!("Failed to acquire lock on SHARED");
    }

    println!("All threads finished.");
}
