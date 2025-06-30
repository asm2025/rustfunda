use anyhow::Result;
use futures::TryStreamExt;
use sqlx::{FromRow, Row};

#[derive(Debug, FromRow)]
struct Message {
    id: i64,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    let db_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::SqlitePool::connect(&db_url).await?;
    // let messages = sqlx::query("SELECT id, message FROM messages")
    //     .map(|row: sqlx::sqlite::SqliteRow| {
    //         let id: i64 = row.get(0);
    //         let message: String = row.get(1);
    //         (id, message)
    //     })
    //     .fetch_all(&pool)
    //     .await?;
    // for (id, message) in messages {
    //     println!("{id}: {message}");
    // }

    println!("Fetch using mapping...");
    let messages = sqlx::query_as::<_, Message>("SELECT id, message FROM messages")
        .fetch_all(&pool)
        .await?;

    for message in messages {
        println!("{message:?}");
    }

    println!("Fetch using stream...");
    let mut stream = sqlx::query_as::<_, Message>("SELECT id, message FROM messages").fetch(&pool);

    while let Some(message) = stream.try_next().await? {
        println!("{message:?}");
    }

    Ok(())
}
