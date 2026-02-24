use std::convert::TryInto;

pub struct Cursor<'a> {
    pub data: &'a [u8],
    pub position: usize,
}

impl<'a> Cursor<'a> {
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
