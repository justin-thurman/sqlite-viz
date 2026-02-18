pub struct Cursor<'a> {
    pub data: &'a [u8],
    pub position: usize,
}

impl<'a> Cursor<'a> {
    pub fn u16_from_be(&mut self) -> u16 {
        let ret = u16::from_be_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        ret
    }
}
