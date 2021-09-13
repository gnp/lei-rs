pub struct DigitsIterator<'a> {
    bytes: &'a [u8],
    scratch: Option<u8>,
}

impl<'a> DigitsIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> DigitsIterator<'a> {
        DigitsIterator {
            bytes,
            scratch: None,
        }
    }
}

impl<'a> Iterator for DigitsIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        match self.scratch {
            Some(d) => {
                self.scratch = None;
                Some(d)
            }
            None => {
                let (d, rest) = self.bytes.split_first()?;
                self.bytes = rest;
                let d = match d {
                    v @ b'0'..=b'9' => v - b'0',
                    v @ b'A'..=b'Z' => v - b'A' + 10u8,
                    _ => panic!("DigitIterator should only be called on pure ASCII uppercase alphanumeric strings")
                };
                if d < 10 {
                    Some(d + b'0')
                } else {
                    self.scratch = Some((d % 10) + b'0');
                    let d = (d / 10) + b'0';
                    Some(d)
                }
            }
        }
    }
}
