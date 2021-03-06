use {
    super::Result,
    bytes::{
        buf::{Buf, BufMut},
        BytesMut,
    },
    serde::de::DeserializeOwned,
    serde::Deserialize,
    std::{collections::VecDeque, convert::TryInto},
};

pub static HEADER_LEN: usize = std::mem::size_of::<u32>();

#[derive(Debug)]
enum State {
    OnHeader,
    OnBody(usize),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Type<T> {
    Single(T),
    Batched(Vec<T>),
}

pub struct SimpleCodec<T> {
    bytes: BytesMut,
    parsed_buf: VecDeque<Result<Type<T>>>,

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

    pub fn drain(&mut self) -> Option<Result<Type<T>>> {
        self.parsed_buf.pop_front()
    }

    fn parse_header(&mut self) -> Option<()> {
        if self.bytes.len() < HEADER_LEN {
            return None;
        }
        self.state = State::OnBody(u32::from_le_bytes(
            (&self.bytes.as_ref()[0..HEADER_LEN]).try_into().unwrap(),
        ) as usize);
        self.bytes.advance(HEADER_LEN);
        self.parse_body()
    }

    fn parse_body(&mut self) -> Option<()> {
        if let State::OnBody(len) = self.state {
            if self.bytes.len() < len {
                return None;
            }

            self.parsed_buf.push_back(
                serde_json::from_slice::<Type<T>>(&self.bytes.as_ref()[..len])
                    .map_err(|e| e.into()),
            );

            self.state = State::OnHeader;
            self.bytes.advance(len);
            return Some(());
        }
        unreachable!();
    }
}
