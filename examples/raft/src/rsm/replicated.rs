use std::cell::Cell;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RepliactedU64 {
    value: Cell<u64>,
    file: Option<mc::File>,
}

impl RepliactedU64 {
    pub async fn new(name: &str) -> Self {
        let file = mc::File::open(name);
        if let Ok(file) = file {
            let mut buf = [0u8; 10];
            let bytes = file.read(&mut buf, 0).await.unwrap();
            assert!(bytes == 0 || bytes == 10);
            let value = Self::value_from_string(&buf[..bytes]);
            Self {
                value: Cell::new(value),
                file: Some(file),
            }
        } else {
            if let Ok(file) = mc::File::create(name) {
                let r = Self {
                    value: Cell::new(0),
                    file: Some(file),
                };
                r.update(0).await.unwrap();
                r
            } else {
                // fs unavailable
                Self {
                    value: Cell::new(0),
                    file: None,
                }
            }
        }
    }

    pub fn value_to_string(value: u64) -> String {
        let s = value.to_string();
        let result = vec!['0' as u8; 10 - s.len()];
        let mut result = String::from_utf8(result).unwrap();
        result.push_str(s.as_str());
        result
    }

    pub fn value_from_string(buf: &[u8]) -> u64 {
        if buf.is_empty() {
            0
        } else {
            let str = std::str::from_utf8(buf).unwrap();
            u64::from_str_radix(str, 10).unwrap()
        }
    }

    pub fn update(&self, new_value: u64) -> mc::JoinHandle<()> {
        if new_value != self.value.get() {
            self.value.set(new_value);
            let s = Self::value_to_string(new_value).into_bytes();
            let file = self.file.clone();
            mc::spawn(async move {
                if let Some(file) = file {
                    let _ = file.write(s.as_slice(), 0).await;
                }
            })
        } else {
            mc::spawn(async {})
        }
    }

    pub fn read(&self) -> u64 {
        self.value.get()
    }

    pub fn increment(&self) -> (u64, mc::JoinHandle<()>) {
        let mut value = self.read();
        value += 1;
        (value, self.update(value))
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
