//! This crate provides a simple TokenBucket object for use in rate-
//! limiting. 
//! 
//! # Short Example Program
//!
//! ```no_run
//! use rate::TokenBucket;
//! use rate::TokenAcquisitionResult;
//! use std::{thread, time};
//! 
//! // Will acquire tokens at the specified rate for the specified duration.
//! // After each acquisition, the AcquisitionResult will be printed.
//! fn run(bucket: &mut TokenBucket, rate: u32, duration: u32) {
//!     for _ in 0..=(rate * duration) {
//!         // Acquire 1 token from the bucket.
//!         let acquisition: TokenAcquisitionResult = bucket.acquire(1.0);
//! 
//!         // Determine the acquisition result.
//!         match acquisition {
//!             Ok(rate)  => println!("rate/allow: {}, true", rate),
//!             Err(rate) => println!("rate/allow: {}, false", rate),
//!         }
//!         
//!         // Sleep for enough time to match the desired rate/second.
//!         thread::sleep(time::Duration::from_micros(
//!             (1000000.0 * (1.0 / rate as f64)) as u64,
//!         ));
//!     }
//! }
//! 
//! fn main() {
//!     // Create the TokenBucket object
//!     let mut token_bucket: TokenBucket = TokenBucket::new(5.0, 100.0);
//! 
//!     // Start of by acquiring 60 tokens per second for 10 seconds.
//!     run(&mut token_bucket, 60, 10);
//! 
//!     // Slow down to 2 tokens per second for 10 seconds.
//!     run(&mut token_bucket, 2, 10);
//! }
//! ```

use std::time::SystemTime;
use std::sync::Mutex;

/// Represents a thread-safe token bucket object.
pub struct TokenBucket {
    // Represents the maximum number of acquisitions per second that
    // this token bucket can sustain. `r` tokens will be added to the
    // bucket each second to sustain acquisitions.
    r:      f64,
    // Represents the "burst" value for the bucket. This is the
    // maximum number of tokens that can be consumed at one time when
    // the bucket is full. It can also be described as the maximum
    // volume of the bucket.
    b:      f64,
    // Represents the number of tokens currently available for
    // acquisition in the bucket.
    tokens: f64,
    // Represents the last time at which one or more tokens was
    // acquired from the bucket.
    last:   SystemTime,
    // A mutex used for locking token acquisition calls on the bucket.
    mux:    Mutex<u32>,
}

/// Represents the acquisition result from a call to 
/// [TokenBucket.acquire()](struct.TokenBucket.html#method.acquire).
///
/// Err() is called if the number of tokens desired is not currently
/// available in the bucket. Otherwise, Ok() is called.
///
/// Both Ok() and Err() will supply the current rate of the Bucket in
/// tokens acquired per second.
pub type TokenAcquisitionResult = Result<f64, f64>;

impl TokenBucket {
    /// Returns a new TokenBucket object.
    ///
    /// # Arguments
    ///
    /// * `r` -  The number of tokens that should be added to the
    ///          bucket every second. This can also be described as
    ///          the maximum rate per second that the bucket can
    ///          sustain before rate limiting.
    ///
    /// * `b` - The "burst" value for the bucket. This is the maximum
    ///         number of tokens that can be consumed at one time when
    ///         the bucket is full. It can also be desribed as the
    ///         maximum volume of the bucket.
    ///
    /// # Example
    ///
    /// ```
    /// let mut tb = TokenBucket::new(5.0, 100.0);
    /// ```
    pub fn new(r: f64, b: f64) -> TokenBucket {
        TokenBucket {
            r,
            b,
            tokens: b,
            last: SystemTime::now(),
            mux: Mutex::new(0u32),
        }
    }

    /// Attempts to acquire `count` tokens from the bucket. 
    ///
    /// Returns a
    /// [TokenAcquisitionResult](type.TokenAcquisitionResult.html).
    ///
    /// Only one acquisition call can be performed per thread at any
    /// given time. Thread safety is maintained by an internal mutex.
    ///
    /// Every time the acquire() function is called:
    ///
    ///    1. `self.r` tokens will be added for every second that has
    ///        elapsed since the last invocation of acquire().
    ///    2. `count` tokens will be removed from the bucket.
    ///    3. The tokens will never exceed the maximum burst value
    ///        configured in `self.b`, nor will it be less than 0.
    ///
    /// ```
    /// self.tokens = min { b, max { 0, tokens + rS - count } }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `count` - The number of tokens to attempt to acquire.
    ///
    /// # Example
    ///
    /// ```
    /// let mut token_bucket = TokenBucket::new(5.0, 100.0);
    /// match token_bucket.acquire(1.0) {
    ///    Ok(rate)  => println!("acquired: rate = {}", rate)
    ///    Err(rate) => println!("rate limited: rate = {}", rate)
    /// }
    /// ```
    pub fn acquire(&mut self, count: f64) -> TokenAcquisitionResult {
        let _guard = self.mux.lock();
        let now = SystemTime::now();
        let duration_ms: u128 = now.duration_since(self.last)
                                   .expect("clock went backwards")
                                   .as_millis();
        let allowed = self.tokens > count;
        self.tokens = self.b.min(
            0f64.max(
                self.tokens
                + (self.r * duration_ms as f64) / 1000 as f64
                - count,
            ),
        );
        let rate :f64 = (1f64 / duration_ms as f64) * 1000 as f64;
        self.last = now;

        if allowed { Ok(rate) } else { Err(rate) }
    }
}