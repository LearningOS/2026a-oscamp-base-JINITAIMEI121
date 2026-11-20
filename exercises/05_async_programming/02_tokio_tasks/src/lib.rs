//! # Tokio Async Tasks
//!
//! In this exercise, you will use `tokio::spawn` to create concurrent asynchronous tasks.
//!
//! ## Concepts
//! - `tokio::spawn` creates asynchronous tasks
//! - `JoinHandle` waits for task completion
//! - Concurrent execution between asynchronous tasks

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

/// Concurrently compute the square of each number in 0..n, collect results and return in order.
///
/// Hint: Create `tokio::spawn` task for each i, collect JoinHandle, await them sequentially.
pub async fn concurrent_squares(n: usize) -> Vec<usize> {
    // Spawn one independent task per index so they are all scheduled concurrently.
    let handles: Vec<JoinHandle<usize>> = (0..n)
        .map(|i| tokio::spawn(async move { i * i }))
        .collect();

    // Await every handle in spawn order, so output is deterministic (index-aligned).
    let mut out = Vec::with_capacity(handles.len());
    for h in handles {
        // A panic inside a spawned task surfaces here as Err; propagate it loudly
        // instead of silently dropping the result.
        out.push(h.await.expect("square task panicked"));
    }
    out
}

/// Concurrently execute multiple "time-consuming" tasks (simulated with sleep), return all results.
/// Each task sleeps `duration_ms` milliseconds and then returns its `task_id`.
///
/// Key: All tasks should execute concurrently, total duration should be close to single task duration, not sum of all tasks.
pub async fn parallel_sleep_tasks(n: usize, duration_ms: u64) -> Vec<usize> {
    // Register every task before awaiting any of them: sleeping happens on the timer
    // driver, so wall-clock cost is ~duration_ms rather than n * duration_ms.
    let handles: Vec<JoinHandle<usize>> = (0..n)
        .map(|task_id| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(duration_ms)).await;
                task_id
            })
        })
        .collect();

    let mut out: Vec<usize> = Vec::with_capacity(handles.len());
    for h in handles {
        out.push(h.await.expect("sleep task panicked"));
    }
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_squares_basic() {
        let result = concurrent_squares(5).await;
        assert_eq!(result, vec![0, 1, 4, 9, 16]);
    }

    #[tokio::test]
    async fn test_squares_zero() {
        let result = concurrent_squares(0).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_squares_one() {
        let result = concurrent_squares(1).await;
        assert_eq!(result, vec![0]);
    }

    #[tokio::test]
    async fn test_parallel_sleep() {
        let start = Instant::now();
        let result = parallel_sleep_tasks(5, 100).await;
        let elapsed = start.elapsed();

        assert_eq!(result, vec![0, 1, 2, 3, 4]);
        // Concurrent execution, total time should be much less than 5 * 100ms
        assert!(
            elapsed.as_millis() < 400,
            "Tasks should run concurrently, took {}ms",
            elapsed.as_millis()
        );
    }
}
