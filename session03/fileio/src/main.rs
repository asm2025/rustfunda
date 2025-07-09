use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::Path,
};
use tokio::{
    fs::File as TkFile,
    io::{AsyncBufReadExt, BufReader as TkBufReader},
};

fn read_lines<P: AsRef<Path>>(filename: P) -> Result<Lines<BufReader<File>>> {
    let filename = filename.as_ref();

    if !filename.exists() {
        let err = format!("{} does not exist.", filename.display());
        println!("{}", &err);
        return Err(anyhow!(err));
    }

    println!("{} exists.", filename.display());

    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

async fn lines_count_async<P: AsRef<Path>>(filename: P) -> Result<usize> {
    let filename = filename.as_ref();

    if !filename.exists() {
        let err = format!("{} does not exist.", filename.display());
        println!("{}", &err);
        return Err(anyhow!(err));
    }

    println!("{} exists.", filename.display());

    let file = TkFile::open(filename).await?;
    let reader = TkBufReader::new(file);
    let mut lines = reader.lines();
    let mut lines_count = 0;

    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            continue;
        }
        lines_count += 1;
    }

    Ok(lines_count)
}

#[tokio::main]
async fn main() -> Result<()> {
    let filename = match std::env::current_dir() {
        Ok(path) => {
            println!("path: {}", path.display());
            path.join("data/warandpeace.txt")
        }
        _ => {
            println!("Could not get the current path!");
            return Err(anyhow!("Could not get the current path!"));
        }
    };

    if let Ok(lines) = read_lines(filename.clone()) {
        let now = std::time::Instant::now();
        let lines_count = lines
            .filter_map(|line| line.ok())
            .filter(|x| !x.is_empty())
            .count();
        println!(
            "Read {} lines in {:.4} seconds.",
            lines_count,
            now.elapsed().as_secs_f64()
        );
    }

    let now = std::time::Instant::now();
    let (c1, c2, ..) = tokio::join!(
        lines_count_async(filename.clone()),
        lines_count_async(filename.clone()),
        lines_count_async(filename.clone()),
        lines_count_async(filename.clone()),
    );
    println!(
        "Read {} lines in {:.4} seconds.",
        c1? + c2?,
        now.elapsed().as_secs_f64()
    );
    Ok(())
}
