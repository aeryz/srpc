//! Streamed reader
//!

use {
    super::{
        codec::{self, SimpleCodec},
        Result,
    },
    futures::stream::Stream,
    serde::de::DeserializeOwned,
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
    tokio::io::{AsyncRead, ReadBuf},
};

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
    type Item = Result<codec::Type<D>>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let self_ref = unsafe { &mut self.get_unchecked_mut() };
        if let Some(Ok(data)) = self_ref.codec.drain() {
            return Poll::Ready(Some(Ok(data)));
        }
        let mut buffer = [0 as u8; 1024];
        loop {
            let mut buf = ReadBuf::new(&mut buffer);
            match self_ref.reader.as_mut().poll_read(cx, &mut buf) {
                Poll::Ready(Ok(_)) => {
                    if buf.filled().is_empty() {
                        return Poll::Ready(None);
                    }
                    self_ref.codec.extend(buf.filled());
                    if let Some(Ok(data)) = self_ref.codec.drain() {
                        return Poll::Ready(Some(Ok(data)));
                    }
                }
                Poll::Ready(Err(e)) => {
                    return Poll::Ready(Some(Err(e.into())));
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

mod tests {
    #[test]
    fn arbitrary() {
        // Generates random bytes and sends them to the reader in arbitrary lengths.
        unimplemented!()
    }

    #[test]
    fn zero_length_body() {
        // Tests if sending header with a length of zero breaks the reader.
        // Zero-header should be sent in the beginning, in the end and somewhere between them.
    }
}
