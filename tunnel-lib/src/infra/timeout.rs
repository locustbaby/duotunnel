use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elapsed;

impl std::fmt::Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "timeout elapsed")
    }
}

impl std::error::Error for Elapsed {}

pin_project! {
    pub struct Timeout<F> {
        #[pin]
        future: F,
        duration: Duration,
        #[pin]
        timer: Option<tokio::time::Sleep>,
    }
}

impl<F> Timeout<F> {
    fn new(duration: Duration, future: F) -> Self {
        Self {
            future,
            duration,
            timer: None,
        }
    }
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Poll::Ready(value) = this.future.poll(cx) {
            return Poll::Ready(Ok(value));
        }
        if this.timer.as_ref().as_pin_ref().is_none() {
            this.timer
                .set(Some(tokio::time::sleep(*this.duration)));
        }
        match this.timer.as_mut().as_pin_mut() {
            Some(timer) => match timer.poll(cx) {
                Poll::Ready(()) => Poll::Ready(Err(Elapsed)),
                Poll::Pending => Poll::Pending,
            },
            None => Poll::Pending,
        }
    }
}

pub fn timeout<F>(duration: Duration, future: F) -> Timeout<F>
where
    F: Future,
{
    Timeout::new(duration, future)
}

pub fn tokio_timeout<F>(
    duration: Duration,
    future: F,
) -> tokio::time::Timeout<F>
where
    F: Future,
{
    tokio::time::timeout(duration, future)
}

pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
