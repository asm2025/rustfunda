use std::{sync::Arc, thread};

fn main() {
    const N_THREADS: usize = 4;
    const LENGTH: u32 = 10000;

    let to_add = Arc::new((1..=LENGTH).collect::<Vec<u32>>());
    let total = thread::scope(|scope| {
        let n_chunks = (to_add.len() + N_THREADS - 1) / N_THREADS;
        let chunks = to_add.chunks(n_chunks);
        let mut handles = vec![];
        println!("Dividing work into {} chunks.", chunks.len());

        for (i, chunk) in chunks.enumerate() {
            let data = Arc::clone(&to_add);
            //let start = (chunk.as_ptr() as usize - data.as_ptr() as usize) / NUM_THREADS;
            let start = i * n_chunks;
            let end = start + chunk.len();
            let handle = scope.spawn(move || {
                let slice = &data[start..end];
                let sum: u32 = slice.iter().sum();
                println!("Chunk[{i}] sum: {sum}");
                sum
            });
            handles.push(handle);
        }

        println!("Waiting for threads to finish...");
        handles.into_iter().map(|h| h.join().unwrap()).sum::<u32>()
    });
    println!("Total: {}", total);

    // Verify the total sum
    if total == (1..=LENGTH).sum::<u32>() {
        println!("The total is correct!");
    } else {
        println!("The total is incorrect.");
    }
}
