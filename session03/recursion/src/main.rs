use async_recursion::*;
use std::{
    future::Future,
    io::{Write, stdout},
    pin::Pin,
};

#[async_recursion]
async fn fibonacci(n: u32) -> u32 {
    if n < 2 {
        return n;
    }

    fibonacci(n - 1).await + fibonacci(n - 2).await
}

async fn one() {
    println!("One");
    stdout().flush().unwrap();
}

async fn two() {
    println!("Two");
    stdout().flush().unwrap();
}

async fn call_one_of_them(n: u32) -> Pin<Box<dyn Future<Output = ()>>> {
    match n {
        1 => Box::pin(one()),
        2 => Box::pin(two()),
        _ => Box::pin(async { panic!("Invalid choice!") }),
    }
}

#[tokio::main]
async fn main() {
    let n = 10;
    println!("fibonacci({n}) = {}", fibonacci(n).await);

    let future = async {
        println!("Hello world!");
    };
    tokio::pin!(future);
    (&mut future).await;

    for i in 0..10 {
        let n = i % 2 + 1;
        let pinned = call_one_of_them(n).await;
        pinned.await;
    }
}
