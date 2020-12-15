use std::convert::TryInto;

#[derive(PartialEq)]
enum State {
    OnHeader,
    OnBody,
    Done,
}

struct SimpleCodec {
    state: State,
    capacity: usize,
    buffer: Vec<u8>,
}

static HEADER_LEN: usize = std::mem::size_of::<usize>();

impl SimpleCodec {
    pub fn new() -> Self {
        SimpleCodec {
            state: State::OnHeader,
            capacity: 0,
            buffer: Vec::new(),
        }
    }

    pub fn from(buffer: &[u8]) -> Self {
        SimpleCodec {
            state: State::OnHeader,
            capacity: 0,
            buffer: buffer.to_vec(),
        }
    }

    fn extend_header(&mut self, buffer: &[u8]) -> Option<usize> {
        if self.buffer.len() >= HEADER_LEN {
            self.capacity = usize::from_be_bytes((&self.buffer[0..HEADER_LEN]).try_into().unwrap());
            self.buffer.drain(0..HEADER_LEN);
            self.state = State::OnBody;
            self.extend_body(buffer, 0)
        } else if buffer.len() + self.buffer.len() >= HEADER_LEN {
            let remaining_len = HEADER_LEN - self.buffer.len();
            self.buffer.extend_from_slice(&buffer[0..remaining_len]);
            self.capacity = usize::from_be_bytes(self.buffer.as_slice().try_into().unwrap());
            self.buffer.clear();
            self.state = State::OnBody;
            self.extend_body(&buffer[remaining_len..buffer.len()], remaining_len)
        } else {
            self.buffer.extend_from_slice(buffer);
            None
        }
    }

    fn extend_body(&mut self, buffer: &[u8], offset: usize) -> Option<usize> {
        let remaining = self.capacity - self.buffer.len();
        let ret = if buffer.len() > remaining {
            self.buffer.extend_from_slice(&buffer[0..remaining]);
            Some(remaining + offset)
        } else {
            self.buffer.extend_from_slice(buffer);
            None
        };
        if self.capacity == self.buffer.len() {
            self.state = State::Done;
        }
        ret
    }

    pub fn extend(&mut self, buffer: &[u8]) -> Option<usize> {
        match self.state {
            State::OnHeader => self.extend_header(buffer),
            State::OnBody => self.extend_body(buffer, 0),
            State::Done => None,
        }
    }

    pub fn can_finalize(&self) -> bool {
        self.state == State::Done
    }

    pub fn finalize(self) -> Vec<u8> {
        self.buffer
    }

    pub fn to_string(&self) -> String {
        format!("expected: {}, data: {}", self.capacity, self.buffer.len())
    }
}

fn main() {}

mod tests {
    use super::*;
    #[test]
    fn cool_test() {
        let mut data = Vec::new();
        let mut lens = Vec::new();
        for _ in 0..1000 {
            let len = rand::random::<usize>() % 10000 as usize;
            let mut new_data = Vec::with_capacity(len);
            lens.push(len);
            new_data.extend_from_slice(&len.to_be_bytes());
            new_data.resize(len + HEADER_LEN, 0);
            new_data
                .iter_mut()
                .skip(HEADER_LEN)
                .for_each(|d| *d = rand::random::<u8>());
            data.extend(new_data);
        }
        let mut codec = SimpleCodec::new();
        let mut cursor = 0;
        let mut lens_index = 0;
        let mut total_len = 0;

        loop {
            let len = match lens.get(lens_index) {
                Some(len) => *len + 8,
                None => return,
            };

            let d_size = if len != cursor {
                rand::random::<usize>() % (data.len() - cursor) + 1
            } else {
                1
            };

            println!("Len: {}, Cursor: {}, DSize: {}", len, cursor, d_size);
            match codec.extend(&data[cursor..cursor + d_size]) {
                Some(new_index) => {
                    assert_eq!(
                        codec.finalize(),
                        &data[total_len + HEADER_LEN..len + total_len - HEADER_LEN]
                    );
                    codec = SimpleCodec::from(&data[cursor..cursor + new_index]);
                    cursor += new_index;
                    lens_index += 1;
                    total_len += len;
                }
                None => {
                    if codec.can_finalize() {
                        assert_eq!(
                            codec.finalize(),
                            &data[total_len + HEADER_LEN..len + total_len - HEADER_LEN]
                        );
                        codec = SimpleCodec::new();
                        println!("Done: {}", lens_index);
                        lens_index += 1;
                        total_len += len;
                    }
                }
            }
            cursor += d_size;
        }
    }
}
