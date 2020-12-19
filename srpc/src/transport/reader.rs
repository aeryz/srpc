use super::simple_codec::*;
use super::Result;
use futures::stream::Stream;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

pub struct Reader<D, R> {
    codec: SimpleCodec<D>,
    reader: Pin<Box<R>>,
}

impl<D, R> Reader<D, R>
where
    D: DeserializeOwned,
    R: AsyncRead + Unpin,
{
    pub fn new(reader: R) -> Self {
        Self {
            codec: SimpleCodec::new(),
            reader: Box::pin(reader),
        }
    }
}

impl<D, R> Stream for Reader<D, R>
where
    D: DeserializeOwned,
    R: AsyncRead + Unpin,
{
    type Item = Result<D>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let self_ref = unsafe { &mut self.get_unchecked_mut() };
        if let Some(Ok(data)) = self_ref.codec.drain() {
            return Poll::Ready(Some(Ok(data)));
        }
        let mut buffer = [0 as u8; 1024];
        let mut buf = ReadBuf::new(&mut buffer);
        loop {
            match self_ref.reader.as_mut().poll_read(cx, &mut buf) {
                Poll::Ready(Ok(_)) => {
                    if buf.filled().is_empty() {
                        return Poll::Ready(None);
                    }
                    self_ref.codec.extend(buf.filled());
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
