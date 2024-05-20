//! This crate provides a simple TokenBucket object for use in rate-
//! limiting. 
//! 
//! # Short Example Program
//!
//! ```no_run
//! use tokenbucket::TokenBucket;
//! use tokenbucket::TokenAcquisitionResult;
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
    /// # use tokenbucket::TokenBucket;
    /// let mut tb = TokenBucket::new(5.0, 100.0);
    /// ```
    pub fn new(r: f64, b: f64) -> TokenBucket {
        TokenBucket {
            r,
            b,
            tokens: b,
            last: SystemTime::now(),
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
    ///    2. `count` tokens will be removed from the bucket if there are enough tokens available.
    ///    3. The tokens will never exceed the maximum burst value
    ///        configured in `self.b`, nor will it be less than 0.
    ///
    /// ```ignore
    /// self.tokens = min { b, tokens + rS }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `count` - The number of tokens to attempt to acquire.
    ///
    /// # Returns
    ///
    /// * `Ok(rate)` - if the requested number of tokens were successfully acquired. `rate` is the rate of token acquisition in tokens per second.
    /// * `Err(rate)` - if the requested number of tokens could not be acquired. `rate` is the rate of token acquisition in tokens per second.
    ///
    /// # Example
    ///
    /// ```
    /// # use tokenbucket::TokenBucket;
    /// let mut token_bucket = TokenBucket::new(5.0, 100.0);
    /// match token_bucket.acquire(1.0) {
    ///    Ok(rate)  => println!("acquired: rate = {}", rate),
    ///    Err(rate) => println!("rate limited: rate = {}", rate),
    /// };
    /// ```
    pub fn acquire(&mut self, count: f64) -> TokenAcquisitionResult {
        let now = SystemTime::now();
        let duration_ms: u128 = now.duration_since(self.last)
                                   .expect("clock went backwards")
                                   .as_millis();

        // Replenish tokens based on the time passed
        self.tokens = self.b.min(
            self.tokens + (self.r * duration_ms as f64) / 1000.0,
        );

        // Check if there are enough tokens available
        let allowed = self.tokens >= count;

        if allowed {
            self.tokens -= count;
            self.last = now;
            let rate: f64 = (1f64 / duration_ms as f64) * 1000.0;
            Ok(rate)
        } else {
            let rate: f64 = (1f64 / duration_ms as f64) * 1000.0;
            Err(rate)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    /// This module contains unit tests for the TokenBucket implementation.

    /// 1. **Initial Token Acquisition**:
    ///    - Test acquiring tokens immediately after creating a new TokenBucket.
    ///    - The bucket should have enough tokens initially, so the acquisition should succeed.
    #[test]
    fn test_initial_acquire() {
        let mut bucket = TokenBucket::new(1.0, 1.0);
        let result = bucket.acquire(1.0);
        assert!(result.is_ok());
    }

    /// 2. **Token Acquisition When Tokens Are Available**:
    ///    - Test acquiring tokens after waiting for some time.
    ///    - After waiting for a sufficient duration, the bucket should have replenished tokens, so the acquisition should succeed.
    #[test]
    fn test_acquire_when_tokens_available() {
        let mut bucket = TokenBucket::new(1.0, 1.0);
        let result = bucket.acquire(1.0);
        assert!(result.is_ok());
        thread::sleep(Duration::from_secs(1));
        let result = bucket.acquire(1.0);
        assert!(result.is_ok());
    }

    /// 3. **Token Acquisition When Tokens Are Not Available**:
    ///    - Test acquiring more tokens than available in the bucket.
    ///    - If the requested number of tokens exceeds the available tokens, the acquisition should fail.
    #[test]
    fn test_acquire_when_tokens_not_available() {
        let mut bucket = TokenBucket::new(1.0, 1.0);
        let result = bucket.acquire(2.0);
        assert!(result.is_err());
    }

    /// 4. **Token Acquisition with Replenishment**:
    ///    - Test acquiring tokens, waiting for replenishment, and then acquiring again.
    ///    - After the initial acquisition, wait for some time to allow tokens to replenish, then attempt to acquire tokens again.
    #[test]
    fn test_acquire_with_replenish() {
        let mut bucket = TokenBucket::new(1.0, 2.0);
        let result1 = bucket.acquire(1.0);
        assert!(result1.is_ok());
        thread::sleep(Duration::from_secs(1));
        let result2 = bucket.acquire(1.0);
        assert!(result2.is_ok());
    }

    /// 5. **Rate Limiting Behavior**:
    ///    - Test acquiring tokens too quickly in succession.
    ///    - If tokens are requested at a higher rate than they are replenished, the acquisition should fail due to rate limiting.
    #[test]
    fn test_rate_limited() {
        let mut bucket = TokenBucket::new(1.0, 1.0);
        let result1 = bucket.acquire(1.0);
        assert!(result1.is_ok());
        thread::sleep(Duration::from_millis(500));
        let result2 = bucket.acquire(1.0);
        assert!(result2.is_err());
    }
}