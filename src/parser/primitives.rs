use std::convert::TryInto;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Unexpected variant truncation")]
    TruncatedVarint,
}

pub struct Cursor<'a> {
    pub data: &'a [u8],
    pub position: usize,
}

impl<'a> Cursor<'a> {
    fn read_byte(&mut self) -> Result<u8, Error> {
        let byte = self
            .data
            .get(self.position)
            .copied()
            .ok_or(Error::UnexpectedEndOfInput)?;
        self.position += 1;
        Ok(byte)
    }

    pub fn read_array<const N: usize>(&mut self) -> [u8; N] {
        // TODO: consider a non-panic version; will have to think where bound checks happen
        let bytes: [u8; N] = self.data[self.position..self.position + N]
            .try_into()
            .unwrap();
        self.position += N;
        bytes
    }

    pub fn u8_from_be(&mut self) -> u8 {
        u8::from_be_bytes(self.read_array::<1>())
    }

    pub fn u16_from_be(&mut self) -> u16 {
        u16::from_be_bytes(self.read_array::<2>())
    }

    pub fn u16_vec_from_be(&mut self, count: usize) -> Vec<u16> {
        // TODO: consider non-panic version; still haven't decided where I want bounds checks to happen
        // TODO: generalize this if we end up duplicating for 3+ types (i.e., u8, u32)
        let start = self.position;
        let end = start + count * 2;
        let vec: Vec<u16> = self.data[start..end]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        self.position += count * 2;
        vec
    }

    pub fn u32_from_be(&mut self) -> u32 {
        u32::from_be_bytes(self.read_array::<4>())
    }

    pub fn read_varint(&mut self) -> Result<i64, Error> {
        let mut value: u64 = 0;
        for i in 0..9 {
            let byte: u64 = self.read_byte().map_err(|_| Error::TruncatedVarint)? as u64;
            if i == 8 {
                // then we're at the end; accumulate all 8 bits and bail out
                value = (value << 8) | byte;
                break;
            }
            let high_bit_set = (byte & 0b1000_0000) != 0;
            let payload = byte & 0b0111_1111;

            value = (value << 7) | payload;

            if !high_bit_set {
                break;
            }
        }
        Ok(value as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_from_be() {
        let data = [0x04, 0x00]; // 1024 in big-endian
        let mut cursor = Cursor {
            data: &data,
            position: 0,
        };

        assert_eq!(cursor.u16_from_be(), 1024);
        assert_eq!(cursor.position, 2);
    }

    #[test]
    fn u16_vec_from_be() {
        let data = [0x04, 0x00, 0x04, 0x00, 0x04, 0x00];
        let mut cursor = Cursor {
            data: &data,
            position: 0,
        };

        assert_eq!(cursor.u16_vec_from_be(2), [1024, 1024]);
        assert_eq!(cursor.position, 4);
    }

    #[test]
    fn read_array() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let mut cursor = Cursor {
            data: &data,
            position: 0,
        };

        assert_eq!(cursor.read_array::<3>(), [0x01, 0x02, 0x03]);
        assert_eq!(cursor.position, 3);

        assert_eq!(cursor.read_array::<3>(), [0x04, 0x05, 0x06]);
        assert_eq!(cursor.position, 6);
    }
}

#[cfg(test)]
mod varint_tests {
    use super::*;

    struct Case {
        name: &'static str,
        data: Vec<u8>,
        expected_result: Result<i64, Error>,
        expected_position: usize,
    }

    #[test]
    fn varint_parsing() {
        let cases = vec![
            Case {
                name: "Single byte, high bit not set",
                data: vec![0b0000_0001],
                expected_result: Ok(1),
                expected_position: 1,
            },
            Case {
                name: "Single byte, zero",
                data: vec![0b0000_0000],
                expected_result: Ok(0),
                expected_position: 1,
            },
            Case {
                name: "Single byte, max 7-bit value",
                data: vec![0b0111_1111],
                expected_result: Ok(127),
                expected_position: 1,
            },
            Case {
                name: "Multiple bytes, high bit not set",
                data: vec![0b0000_0001, 0b0000_0001],
                expected_result: Ok(1),
                expected_position: 1,
            },
            Case {
                name: "Two bytes, high bit set",
                data: vec![0b1000_0001, 0b0000_0001],
                expected_result: Ok(129),
                expected_position: 2,
            },
            Case {
                name: "Two bytes, smallest value requiring continuation",
                data: vec![0b1000_0001, 0b0000_0000],
                expected_result: Ok(128),
                expected_position: 2,
            },
            Case {
                name: "Two bytes, max 14-bit value",
                data: vec![0b1111_1111, 0b0111_1111],
                expected_result: Ok(16383), // (2^14 - 1)
                expected_position: 2,
            },
            Case {
                name: "Three bytes",
                data: vec![0b1000_0001, 0b1000_0000, 0b0000_0001],
                expected_result: Ok(16385),
                expected_position: 3,
            },
            Case {
                name: "Eight bytes (56-bit value, no ninth byte)",
                data: vec![
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0111_1111,
                ],
                expected_result: Ok((1i64 << 56) - 1),
                expected_position: 8,
            },
            Case {
                name: "Nine bytes, i64::MAX",
                data: vec![
                    0b1011_1111, // Note the leading bit (after accounting for continuation bit) is unset
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                ],
                expected_result: Ok(i64::MAX),
                expected_position: 9,
            },
            Case {
                name: "Ten bytes provided, but only first nine are read",
                data: vec![
                    0b1111_1111, // 1 (numbering for clarity)
                    0b1111_1111, // 2
                    0b1111_1111, // 3
                    0b1111_1111, // 4
                    0b1111_1111, // 5
                    0b1111_1111, // 6
                    0b1111_1111, // 7
                    0b1111_1111, // 8
                    0b1111_1111, // 9th byte (full 8 bits consumed)
                    0b0000_0001, // 10th byte (should NOT be read)
                ],
                expected_result: Ok(-1), // All bits set => two’s complement -1
                expected_position: 9,
            },
            Case {
                name: "Nine bytes, i64::MIN",
                data: vec![
                    0b1100_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b0000_0000,
                ],
                expected_result: Ok(i64::MIN),
                expected_position: 9,
            },
            Case {
                name: "Corrupted varint: continuation bit set but no more bytes",
                data: vec![0b1000_0001],
                expected_result: Err(Error::TruncatedVarint),
                expected_position: 1,
            },
            Case {
                name: "Corrupted varint: truncated after multiple continuation bytes",
                data: vec![0b1000_0001, 0b1000_0000],
                expected_result: Err(Error::TruncatedVarint),
                expected_position: 2,
            },
        ];
        for case in cases {
            let mut cursor = Cursor {
                data: &*case.data,
                position: 0,
            };
            let result = cursor.read_varint();
            assert_eq!(
                result, case.expected_result,
                "unexpected result for case {}",
                case.name
            );
            assert_eq!(
                cursor.position, case.expected_position,
                "unexpected ending cursor position for case {}",
                case.name
            )
        }
    }
}
