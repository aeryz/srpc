use {
    super::Result,
    bytes::{
        buf::{Buf, BufMut},
        BytesMut,
    },
    serde::de::DeserializeOwned,
    std::{collections::VecDeque, convert::TryInto},
};

static HEADER_LEN: usize = 8;

#[derive(Debug)]
enum State {
    OnHeader,
    OnBody(usize),
}

pub struct SimpleCodec<T> {
    bytes: BytesMut,
    parsed_buf: VecDeque<Result<T>>,

    state: State,
}

impl<T> SimpleCodec<T>
where
    T: DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            bytes: BytesMut::new(),
            parsed_buf: VecDeque::new(),
            state: State::OnHeader,
        }
    }

    pub fn extend(&mut self, data: &[u8]) {
        self.bytes.put(data);
        loop {
            if let None = match self.state {
                State::OnHeader => self.parse_header(),
                State::OnBody(_) => self.parse_body(),
            } {
                break;
            }
        }
    }

    pub fn drain(&mut self) -> Option<Result<T>> {
        self.parsed_buf.pop_front()
    }

    fn parse_header(&mut self) -> Option<()> {
        if self.bytes.len() < HEADER_LEN {
            return None;
        }
        self.state = State::OnBody(usize::from_le_bytes(
            (&self.bytes.as_ref()[0..HEADER_LEN]).try_into().unwrap(),
        ));
        self.parse_body()
    }

    fn parse_body(&mut self) -> Option<()> {
        if let State::OnBody(len) = self.state {
            if self.bytes.len() < len + HEADER_LEN {
                return None;
            }
            self.parsed_buf.push_back(
                serde_json::from_slice::<T>(&self.bytes.as_ref()[HEADER_LEN..HEADER_LEN + len])
                    .map_err(|e| e.into()),
            );
            self.state = State::OnHeader;
            self.bytes.advance(HEADER_LEN + len);
            return Some(());
        }
        unreachable!();
    }
}
