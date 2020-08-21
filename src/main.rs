use tokenbucket::TokenBucket;
use tokenbucket::TokenAcquisitionResult;
use std::{thread, time};

// Will acquire tokens at the specified rate for the specified duration.
// After each acquisition, the AcquisitionResult will be printed.
fn run(bucket: &mut TokenBucket, rate: u32, duration: u32) {
    for _ in 0..=(rate * duration) {
        // Acquire 1 token from the bucket.
        let acquisition: TokenAcquisitionResult = bucket.acquire(1.0);

        // Determine the acquisition result.
        match acquisition {
            Ok(rate)  => println!("rate/allow: {}, true", rate),
            Err(rate) => println!("rate/allow: {}, false", rate),
        }
        
        // Sleep for enough time to match the desired rate/second.
        thread::sleep(time::Duration::from_micros(
            (1000000.0 * (1.0 / rate as f64)) as u64,
        ));
    }
}

fn main() {
    // Create the TokenBucket object
    let mut token_bucket: TokenBucket = TokenBucket::new(5.0, 100.0);

    // Start of by acquiring 60 tokens per second for 10 seconds.
    run(&mut token_bucket, 60, 10);

    // Slow down to 2 tokens per second for 10 seconds.
    run(&mut token_bucket, 2, 10);
}