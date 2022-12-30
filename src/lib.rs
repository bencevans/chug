//! A library for estimating the time remaining until a task is completed.
//!
//! # Example
//!
//! ```
//! use chug::Chug;
//! 
//! let mut chug = Chug::new(10, 100);
//! 
//! for _ in 0..100 {
//!     let formatted_eta = match chug.eta() {
//!         Some(eta) => {
//!             let eta_secs = eta.as_secs();
//!             let eta_millis = eta.subsec_millis();
//!             format!("ETA: {}.{:03}", eta_secs, eta_millis)
//!         }
//!         None => "ETA: None".to_string(),
//!     };
//!     println!("{}", formatted_eta);
//! 
//!     // Do some work...
//! 
//!     chug.tick();
//! }
//! ```
//!

use std::time::{Duration, Instant};

/// A leaky bucket.
///
/// The bucket holds a maximum of `max` items. When a new item is added, the
/// oldest item is removed.
struct LeakyBucket {
    _last_n: Vec<Instant>,
    _max: usize,
}

impl LeakyBucket {
    /// Creates a new `LeakyBucket` instance.
    ///
    /// `max` is the maximum number of items to keep track of.
    pub fn new(max: usize) -> Self {
        Self {
            _last_n: Vec::with_capacity(max),
            _max: max,
        }
    }

    /// Adds a new item to the bucket.
    pub fn insert(&mut self, now: Instant) {
        if self._last_n.len() == self._max {
            self._last_n.remove(0);
        }

        self._last_n.push(now);
    }

    /// Returns the number of items in the bucket.
    ///
    /// This is the number of items that have been added to the bucket, not the
    /// maximum number of items the bucket can hold.
    pub fn len(&self) -> usize {
        self._last_n.len()
    }

    /// Returns reference to the vector of items in the bucket.
    pub fn items(&self) -> &Vec<Instant> {
        &self._last_n
    }
}

pub struct Chug {
    _bucket: LeakyBucket,
    _current_work: usize,
    _total_work: usize,
}

impl Chug {
    /// Creates a new `Chug` instance.
    ///
    /// `max` is the maximum number of units of work to keep track of.
    /// `total_work` is the total number of units of work to be completed.
    ///
    pub fn new(max: usize, total_work: usize) -> Self {
        Self {
            _bucket: LeakyBucket::new(max),
            _current_work: 0,
            _total_work: total_work,
        }
    }

    /// Informs a unit of work has been completed.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self._current_work += 1;
        self._bucket.insert(now);
    }

    /// Estimates the time remaining until the work is completed.
    ///
    /// The estimate is based on the average time between the last `max` units of
    /// work.
    ///
    /// Returns `None` if the work is completed or if there is not enough data to
    /// estimate the time remaining. Otherwise, returns the estimated time
    /// remaining as a `Duration`.
    ///
    pub fn eta(&self) -> Option<Duration> {
        if self._bucket.len() < 2 {
            return None;
        }

        let average_between = {
            let mut sum = 0;
            let mut last = None;
            for now in self._bucket.items() {
                if let Some(last) = last {
                    sum += now.duration_since(last).as_millis() as usize;
                }
                last = Some(*now);
            }
            sum / self._bucket.len()
        };

        if self._current_work > self._total_work {
            return None;
        }

        let remaining = self._total_work - self._current_work;

        if remaining == 0 {
            None
        } else {
            let eta = average_between * remaining;
            Some(std::time::Duration::from_millis(eta as u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaky_bucket() {
        let mut bucket = LeakyBucket::new(10);
        assert_eq!(bucket.len(), 0);

        for i in 0..10 {
            bucket.insert(Instant::now());
            assert_eq!(bucket.len(), i + 1);
        }

        for _ in 0..10 {
            bucket.insert(Instant::now());
            assert_eq!(bucket.len(), 10);
        }
    }

    #[test]
    fn test_empty() {
        let chug = Chug::new(10, 100);
        assert_eq!(chug.eta(), None);
    }

    #[test]
    fn test_completed() {
        let mut chug = Chug::new(10, 100);
        for _ in 0..100 {
            chug.tick();

            // sleep for 1ms
            std::thread::sleep(std::time::Duration::from_millis(10));

            match chug.eta() {
                Some(eta) => {
                    println!("ETA: {}", eta.as_secs());
                }
                None => {
                    println!("ETA: None");
                }
            }
        }
        assert_eq!(chug.eta(), None);
    }

    #[test]
    fn test_smaller_than_max() {
        let mut chug = Chug::new(10, 100);
        for _ in 0..4 {
            chug.tick();
        }
        // check is an instant
        assert!(chug.eta().is_some())
    }

    #[test]
    fn test_larger_than_max() {
        let mut chug = Chug::new(10, 100);
        for _ in 0..30 {
            chug.tick();
        }
        assert!(chug.eta().is_some())
    }

    #[test]
    fn test_just_under_max() {
        let mut chug = Chug::new(10, 100);
        for _ in 0..9 {
            chug.tick();
        }
        assert!(chug.eta().is_some())
    }

    #[test]
    fn test_just_over_total() {
        let mut chug = Chug::new(10, 100);
        for _ in 0..200 {
            chug.tick();
        }
        assert!(chug.eta().is_none())
    }

    #[test]
    fn test_just_under_total() {
        let mut chug = Chug::new(10, 100);
        for _ in 0..99 {
            chug.tick();
        }
        assert!(chug.eta().is_some())
    }
}
