//! Re-exports asynchronous structures from `tokio`, `async-std`, `pollster` and `futures`

#[cfg(any(feature = "async-monoio", feature = "async-pollster"))]
mod pollster_utils;

pub use futures::future::{join_all, select_all};

#[cfg(feature = "async-tokio")]
pub use tokio::{
    sync::{
        Mutex,
        MutexGuard,
        oneshot
    },
    task,
    task::yield_now
};

#[cfg(feature = "async-astd")]
pub use async_std::{
    sync::{ Mutex, MutexGuard },
    task,
    task::yield_now
};

#[cfg(feature = "async-monoio")]
pub mod sync {
    pub use futures::lock::{Mutex, MutexGuard};
}

#[cfg(feature = "async-monoio")]
pub mod task {
    pub use monoio::spawn;
    pub use monoio::task::JoinHandle;
    pub use crate::async_utils::pollster_utils::yield_now;
}

#[cfg(feature = "async-astd")]
pub use futures::channel::oneshot;

#[cfg(feature = "async-pollster")]
pub use crate::async_utils::pollster_utils::testing_sleep;

#[cfg(any(feature = "async-monoio", feature = "async-pollster"))]
pub use crate::async_utils::pollster_utils::yield_now;

use std::future::Future;

#[cfg(feature = "async-astd")]
/// Just block on your `Future`
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// async_std::task::block_on(fut);
/// ```
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static
{
    async_std::task::block_on(fut)
}

#[cfg(feature = "async-tokio")]
/// Just block on your `Future`
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// tokio::runtime::Builder::new_current_thread()
///     .enable_time()
///     .build()
///     .unwrap()
///     .block_on(fut);
/// ```
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static
{
    tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap().block_on(fut)
}

#[cfg(feature = "async-pollster")]
/// Just block on your `Future`
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// pollster::block_on(fut);
/// ```
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static
{
    pollster::block_on(fut)
}

#[cfg(feature = "async-monoio")]
/// Just block on your `Future`
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// monoio::RuntimeBuilder::new()
///     .enable_timer()
///     .build()
///     .unwrap()
///     .block_on(fut)
/// ```
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static,
          R: 'static
{
    monoio::RuntimeBuilder::new().enable_timer().build().unwrap().block_on(fut)
}

#[cfg(feature = "async-tokio")]
/// Just sleep for a while
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// tokio::time::sleep(duration).await
/// ```
pub async fn testing_sleep(duration: std::time::Duration) {
    tokio::time::sleep(duration).await
}

#[cfg(feature = "async-astd")]
/// Just sleep for a while
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// async_std::task::sleep(duration).await
/// ```
pub async fn testing_sleep(duration: std::time::Duration) {
    async_std::task::sleep(duration).await
}

#[cfg(feature = "async-monoio")]
/// Just sleep for a while
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// monoio::time::sleep(duration).await
/// ```
pub async fn testing_sleep(duration: std::time::Duration) {
    monoio::time::sleep(duration).await
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use crate::async_utils::{block_on_future, testing_sleep, yield_now};

    #[test]
    fn test_basic_rt() {
        async fn test_inner() {
            testing_sleep(Duration::from_secs(1)).await;
            yield_now().await;
            testing_sleep(Duration::from_secs(1)).await;
        }

        block_on_future(test_inner())
    }
}
