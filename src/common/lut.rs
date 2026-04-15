pub const fn u8_to_index(s: &[u8]) -> [Option<u8>; 256] {
    let mut out: [Option<u8>; 256] = [None; 256];
    if s.len() > 256 {
        panic!("lut input too big");
    }
    let mut i = 0;
    while i < s.len() {
        out[s[i] as usize] = Some(i as u8);
        i += 1;
    }
    out
}

pub struct AsciiLut {
    u8_to_index: [u8; 128],
    index_to_u8: [u8; 128],
}

impl AsciiLut {
    const fn empty() -> AsciiLut {
        AsciiLut {
            u8_to_index: [255u8; 128],
            index_to_u8: [255u8; 128],
        }
    }

    const fn add_alphabet(&mut self, chars: &str) {
        let bytes = chars.as_bytes();
        if bytes.len() > 127 {
            panic!("alphabet too big");
        }
        let n = bytes.len();
        let mut i = 0;
        while i < n {
            let b = bytes[i];
            if b > 127 {
                panic!("invalid byte at index");
            }
            self.u8_to_index[b as usize] = i as u8;
            self.index_to_u8[i as usize] = b;
            i += 1;
        }
    }

    pub const fn from_alphabet(chars: &str) -> AsciiLut {
        let mut res = AsciiLut::empty();
        res.add_alphabet(chars);
        res
    }

    pub const fn from_alphabet_case_insensitive(chars: &str) -> AsciiLut {
        let mut buf: [u8; 128] = [0; 128];
        let string: &mut str = {
            let bytes = chars.as_bytes();
            let n = chars.len();
            let mut i = 0;
            while i < n {
                buf[i] = bytes[i];
                i += 1;
            }
            let (a, _) = buf.split_at_mut(n);
            match str::from_utf8_mut(a) {
                Ok(x) => x,
                Err(_) => panic!(),
            }
        };
        let mut res = AsciiLut::empty();
        string.make_ascii_lowercase();
        res.add_alphabet(string);
        string.make_ascii_uppercase();
        res.add_alphabet(string);
        res.add_alphabet(chars);
        res
    }

    pub fn decode(&self, c: char) -> Option<u8> {
        let c = c as usize;
        if c >= 128 {
            return None;
        }
        match self.u8_to_index[c] {
            255 => None,
            i => Some(i),
        }
    }

    pub fn encode(&self, i: u8) -> Option<char> {
        if i >= 128 {
            return None;
        }
        match self.index_to_u8[i as usize] {
            255 => None,
            c => Some(c as char),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::common::lut::AsciiLut;

    #[test]
    pub fn test_lut() {
        let lut = AsciiLut::from_alphabet("abc");

        assert_eq!(lut.decode('a'), Some(0));
        assert_eq!(lut.decode('b'), Some(1));
        assert_eq!(lut.decode('c'), Some(2));
        assert_eq!(lut.decode('A'), None);
        assert_eq!(lut.decode('\u{2014}'), None);

        assert_eq!(lut.encode(0), Some('a'));
        assert_eq!(lut.encode(1), Some('b'));
        assert_eq!(lut.encode(2), Some('c'));
        assert_eq!(lut.encode(3), None);
        assert_eq!(lut.encode(128), None);
    }

    #[test]
    pub fn test_lut_case_insensitive() {
        let lut = AsciiLut::from_alphabet_case_insensitive("abc");

        assert_eq!(lut.decode('a'), Some(0));
        assert_eq!(lut.decode('b'), Some(1));
        assert_eq!(lut.decode('c'), Some(2));
        assert_eq!(lut.decode('A'), Some(0));
        assert_eq!(lut.decode('\u{2014}'), None);

        assert_eq!(lut.encode(0), Some('a'));
        assert_eq!(lut.encode(1), Some('b'));
        assert_eq!(lut.encode(2), Some('c'));
        assert_eq!(lut.encode(3), None);
        assert_eq!(lut.encode(128), None);
    }
}
