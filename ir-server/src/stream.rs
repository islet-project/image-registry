use bytes::Bytes;
use core::pin::Pin;
use futures_core::ready;
use futures_core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct SkipBytes<St>
    {
        #[pin]
        stream: St,
        remaining: usize,
    }
}

impl<St: Stream> SkipBytes<St>
{
    fn new(stream: St, n: usize) -> Self
    {
        Self {
            stream,
            remaining: n,
        }
    }
}

pub trait StreamBytesExt: Stream
{
    fn skip_bytes(self, n: usize) -> SkipBytes<Self>
    where
        Self: Sized,
    {
        SkipBytes::new(self, n)
    }
}

impl<St> Stream for SkipBytes<St>
where
    St: Stream<Item = std::io::Result<Bytes>>,
{
    type Item = std::io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>
    {
        let mut this = self.project();

        while *this.remaining > 0 {
            let elem = ready!(this.stream.as_mut().poll_next(cx));

            let Some(elem) = elem else {
                return Poll::Ready(None);
            };
            let Ok(elem) = elem else {
                return Poll::Ready(Some(elem));
            };

            if elem.len() > *this.remaining {
                let start = *this.remaining;
                *this.remaining = 0;
                return Poll::Ready(Some(Ok(elem.slice(start..))));
            } else {
                *this.remaining -= elem.len();
            }
        }

        this.stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let (lower, upper) = self.stream.size_hint();

        let lower = lower.saturating_sub(self.remaining);
        let upper = upper.map(|x| x.saturating_sub(self.remaining));

        (lower, upper)
    }
}

impl<T: Stream> StreamBytesExt for T {}
