mod myerrors;

use anyhow::anyhow;
use myerrors::{MyErrors, Result as MyResult};

fn raise_my_error() -> MyResult<()> {
    Err(MyErrors::NotImplemented)
}

fn raise_my_error2() -> MyResult<()> {
    Err(MyErrors::InvalidOperation(
        "Oh no! This is a test error.".to_string(),
    ))
}

fn raise_my_io_error() -> MyResult<()> {
    Err(MyErrors::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Oh no! This is a I/O test error in MyErrors.",
    )))
}

fn raise_anyhow_error() -> anyhow::Result<()> {
    Err(anyhow!("Oh no! This is an anyhow test error."))
}

fn raise_error() -> std::result::Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Oh no! This is an I/O test error.",
    ))
}

fn main() {
    println!();
    println!("MyErrors use thiserror crate for creating custom errors enum type.");
    println!("It's best used for libraries so applications can catch error by type.");
    println!();

    if let Err(ex) = raise_my_error() {
        println!("Got a MyError error: {}", ex);
    }

    if let Err(ex) = raise_my_error2() {
        println!("Got a MyError error: {}", ex);
    }

    if let Err(ex) = raise_my_io_error() {
        println!("Got a MyError error of type io: {}", ex);
    }

    println!();
    println!("anyhow crate cares about displaying the error message rather than the error type.");
    println!("It's best used for applications but not for libraries.");

    if let Err(ex) = raise_anyhow_error() {
        println!("Got an error of type anyhow error: {}", ex);
    }

    if let Err(ex) = raise_error() {
        println!("Got a regular error of type io: {}", ex);
    }
}
