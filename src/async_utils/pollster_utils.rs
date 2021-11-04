use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct YieldFuture {
    yielded: bool
}

impl Default for YieldFuture {
    fn default() -> Self {
        Self { yielded: false }
    }
}

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        if this.yielded {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            this.yielded = true;
            Poll::Pending
        }
    }
}

pub fn yield_now() -> YieldFuture {
    YieldFuture::default()
}

pub struct SleepFuture {
    sleep_duration: Duration,
    sleep_start: Instant
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        if this.sleep_start.elapsed() >= this.sleep_duration {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn testing_sleep(duration: Duration) -> SleepFuture {
    SleepFuture {
        sleep_duration: duration,
        sleep_start: Instant::now()
    }
}
