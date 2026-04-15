use crate::common::lut;

static B27_ALPHABET: &'static str = "0123456789abcdefghijklmnopq";
static B27_LUT: lut::AsciiLut = lut::AsciiLut::from_alphabet_case_insensitive(B27_ALPHABET);

pub fn ib3_to_sb27be<It>(digits: It) -> Result<String, &'static str>
where
    It: Iterator<Item = u8>,
{
    let mut out = String::new();

    let mut num = 0;
    let mut pass = 0;
    let tri = [9, 3, 1];
    for d in digits {
        if d >= 3 {
            return Err("invalid digit in input");
        }
        pass += d * tri[num];
        num += 1;
        if num == 3 {
            let c = B27_LUT.encode(pass).unwrap();
            out.push(c);
            num = 0;
            pass = 0;
        }
    }
    if num > 0 {
        out.push(B27_LUT.encode(pass).unwrap());
    }

    Ok(out)
}

pub fn sb27be_to_ib3<It>(chars: It) -> Result<Vec<u8>, &'static str>
where
    It: Iterator<Item = char>,
{
    let mut out = Vec::new();

    for c in chars {
        let d = match B27_LUT.decode(c) {
            Some(d) => d,
            None => return Err("invalid character in input"),
        };
        let d0 = d % 3;
        let d1 = (d / 3) % 3;
        let d2 = (d / 9) % 3;
        out.push(d2);
        out.push(d1);
        out.push(d0);
    }

    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(
            ib3_to_sb27be([0, 1, 0, 0, 2, 0].into_iter()),
            Ok("36".to_owned())
        );
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            sb27be_to_ib3("36".chars().into_iter()),
            Ok(vec![0, 1, 0, 0, 2, 0])
        );
        assert_eq!(
            sb27be_to_ib3("Ab".chars().into_iter()),
            Ok(vec![1, 0, 1, 1, 0, 2])
        );
    }
}
