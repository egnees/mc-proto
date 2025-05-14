#[derive(Default)]
pub struct RepliactedU64 {
    value: u64,
    file: Option<mc::File>,
}

impl RepliactedU64 {
    pub fn value_to_string(value: u64) -> String {
        let s = value.to_string();
        let result = vec!['0' as u8; 10 - s.len()];
        let mut result = String::from_utf8(result).unwrap();
        result.push_str(s.as_str());
        result
    }

    pub fn value_from_string(buf: &[u8]) -> u64 {
        let str = std::str::from_utf8(buf).unwrap();
        u64::from_str_radix(str, 10).unwrap()
    }

    pub async fn update(&mut self, new_value: u64) -> mc::FsResult<()> {
        let s = Self::value_to_string(new_value).into_bytes();
        self.file
            .get_or_insert(mc::File::open("current_term.txt")?)
            .write(s.as_slice(), 0)
            .await
            .map(|bytes| {
                assert_eq!(bytes, 10);

                // update value
                self.value = new_value;
            })
    }

    pub async fn read(&mut self) -> mc::FsResult<u64> {
        if self.file.is_none() {
            self.file = Some(mc::File::open("current_term.txt")?);
        }
        Ok(self.value)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::RepliactedU64;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn value_to_string() {
        let x = 5515;
        let s = RepliactedU64::value_to_string(x);
        assert_eq!(s, "0000005515");
    }

    ////////////////////////////////////////////////////////////////////////////////'

    #[test]
    fn value_from_string() {
        let x = RepliactedU64::value_from_string("0000005515".as_bytes());
        assert_eq!(x, 5515);
    }
}
