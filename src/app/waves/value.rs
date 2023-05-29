use std::ops::{Index, IndexMut};

use annotate_snippets::{
    display_list::FormatOptions,
    snippet::{AnnotationType, Slice, Snippet, SourceAnnotation},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BitValue {
    bits_size: usize,
    data: [u64; Self::INNER_LEN],
    neg: bool,
    lsb: bool,
}

enum IntBase {
    B2,
    B8,
    B10,
    B16,
}

impl IntBase {
    fn get_size(&self) -> usize {
        match self {
            IntBase::B2 => 64,
            IntBase::B8 => 21,
            IntBase::B10 => 19,
            IntBase::B16 => 16,
        }
    }
    fn get_radix(&self) -> u32 {
        match self {
            IntBase::B2 => 2,
            IntBase::B8 => 8,
            IntBase::B10 => 10,
            IntBase::B16 => 16,
        }
    }

    fn get_max(&self) -> usize {
        match self {
            IntBase::B2 => 64,
            IntBase::B8 => 23,
            IntBase::B10 => 21,
            IntBase::B16 => 16,
        }
    }

    fn print(&self, v: u64, bits: usize) -> String {
        match self {
            IntBase::B2 => format!("{:0>1$b}", v, bits),
            IntBase::B8 => format!("{:0>1$o}", v, bits),
            IntBase::B10 => format!("{:0>1$}", v, bits),
            IntBase::B16 => format!("{:0>1$x}", v, bits),
        }
    }
}

impl BitValue {
    pub const INNER_LEN: usize = 8;
    pub const BYTE: usize = 64;
    pub const BITS: usize = Self::INNER_LEN * Self::BYTE;

    pub fn new(size: usize) -> Self {
        BitValue {
            bits_size: size,
            data: [0u64; Self::INNER_LEN],
            neg: false,
            lsb: true,
        }
    }

    pub fn parse_from<'a>(&'a mut self, s: &'a str) -> Result<(), Snippet<'a>> {
        let mut snippet = Snippet {
            title: None,
            footer: vec![],
            slices: vec![Slice {
                source: s,
                line_start: 0,
                origin: None,
                fold: false,
                annotations: Vec::with_capacity(1),
            }],
            opt: FormatOptions {
                color: true,
                anonymized_line_numbers: true,
                margin: None,
            },
        };
        let mut chars: Vec<_> = s.chars().collect();
        let mut i = 0;
        self.neg = if let Some(n) = chars.first() {
            match *n {
                '-' => {
                    i += 1;
                    true
                }
                '+' => {
                    i += 1;
                    false
                }
                _ => false,
            }
        } else {
            snippet.slices[0].annotations.push(SourceAnnotation {
                range: (0, 0),
                label: "At least must be 1 symbol",
                annotation_type: AnnotationType::Error,
            });
            return Err(snippet);
        };

        let base = if let Some(n) = chars.get(i..i + 2) {
            if n[0] == '0' {
                i += 2;
                match n[1].to_ascii_lowercase() {
                    'b' => IntBase::B2,
                    'o' => IntBase::B8,
                    'x' => IntBase::B16,
                    c => {
                        if c.is_alphabetic() {
                            snippet.slices[0].annotations.push(SourceAnnotation {
                                range: (0, i),
                                label: "expected base type: 0b, -0o, 0x, 99. Aplabetic found here",
                                annotation_type: AnnotationType::Error,
                            });
                            return Err(snippet);
                        }
                        i -= 2;
                        IntBase::B10
                    }
                }
            } else {
                IntBase::B10
            }
        } else {
            IntBase::B10
        };

        let step = base.get_size();
        let mut errors = false;
        let mut inner = [0u64; Self::INNER_LEN];
        chars.reverse();
        chars[0..(chars.len() - i)]
            .chunks(step)
            .enumerate()
            .for_each(|(is, v)| {
                let s: String = v.iter().rev().collect();
                match u64::from_str_radix(&s, base.get_radix()) {
                    Ok(nw) => {
                        inner[is] = nw;
                    }
                    Err(e) => {
                        snippet.slices[0].annotations.push(SourceAnnotation {
                            range: (i + is * step, i + is * step + s.len()),
                            label: match e.kind() {
                                std::num::IntErrorKind::Empty => {
                                    "cannot parse integer from empty string"
                                }
                                std::num::IntErrorKind::InvalidDigit => {
                                    "invalid digit found in string"
                                }
                                std::num::IntErrorKind::PosOverflow => {
                                    "number too large to fit in target type"
                                }
                                std::num::IntErrorKind::NegOverflow => {
                                    "number too small to fit in target type"
                                }
                                std::num::IntErrorKind::Zero => {
                                    "number would be zero for non-zero type"
                                }
                                _ => todo!(),
                            },
                            annotation_type: AnnotationType::Error,
                        });
                        errors = true;
                    }
                };
            });
        if errors {
            Err(snippet)
        } else {
            self.data = inner;
            Ok(())
        }
    }

    pub fn set_zero(&mut self) {
        self.data = [0u64; Self::INNER_LEN];
    }

    pub fn set_size(&mut self, size: usize) -> Result<(), ()> {
        if size > Self::BITS {
            return Err(());
        };
        if size >= self.bits_size {
            self.bits_size = size;
        } else {
            let byte = size / Self::BYTE;
            let bit = size % Self::BYTE;
            let mask = self.get_mask(size);
            self.data[byte] &= mask;
            for i in byte + 1..Self::BYTE {
                self.data[i] = 0;
            }
        }
        Ok(())
    }

    pub fn data(&self) -> &[u64; Self::INNER_LEN] {
        &self.data
    }

    #[inline(always)]
    pub fn bool(&self) -> bool {
        (self.data[0] & 0b1) as u8 != 0
    }

    #[inline(always)]
    pub fn neg_bool(&mut self) {
        self.data[0] = !self.data[0] & 0b1;
    }

    #[inline(always)]
    fn get_mask(&self, size: usize) -> u64 {
        let bit = size % Self::BYTE;
        if bit == 0 {
            !0u64
        } else {
            !(-0b1i64 << bit) as u64
        }
    }

    pub fn to_bin(&self) -> String {
        return self.print_base(IntBase::B2);

        let mut s = String::new();
        if self.lsb {
            let mut last = (self.bits_size / Self::BYTE).saturating_sub(1);
            let bit = self.bits_size % Self::BYTE;
            // if bit == 0{
            //     last += 1;
            // }
            s += &format!(
                "{:0>1$b}",
                if self.neg {
                    (!self.data[last] & self.get_mask(self.bits_size)) + 1
                } else {
                    self.data[last] & self.get_mask(self.bits_size)
                },
                if bit == 0 { Self::BYTE } else { bit }
            );
            for b in (0..last).rev() {
                if self.neg {
                    s += &format!("{:0>64b}", !self.data[b] + 1);
                } else {
                    s += &format!("{:0>64b}", self.data[b]);
                }
            }
        }
        s
    }

    pub fn to_oct(&self) -> String {
        self.print_base(IntBase::B8)
    }

    pub fn to_dec(&self) -> String {
        self.print_base(IntBase::B10)
    }

    pub fn to_hex(&self) -> String {
        self.print_base(IntBase::B16)
    }

    fn print_base(&self, base: IntBase) -> String {
        let mut s = String::new();
        if self.lsb {
            let last = (self.bits_size / Self::BYTE).saturating_sub(1);
            let bit = self.bits_size % Self::BYTE;
            // if bit == 0{
            //     last += 1;
            // }
            s += &base.print(
                if self.neg {
                    (!self.data[last] & self.get_mask(self.bits_size)) + 1
                } else {
                    self.data[last] & self.get_mask(self.bits_size)
                },
                if bit == 0 { base.get_max() } else { bit },
            );

            for b in (0..last).rev() {
                if self.neg {
                    s += &base.print(!self.data[b] + 1, base.get_size());
                } else {
                    s += &base.print(self.data[b], base.get_size());
                }
            }
        }
        s
    }
}

impl Index<usize> for BitValue {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        if index > Self::BITS {
            panic!("Out of range index {index}. Max: {}", Self::BITS);
        }
        &self.data[index]
    }
}

impl IndexMut<usize> for BitValue {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > Self::BITS {
            panic!("Out of range index {index}. Max: {}", Self::BITS);
        }
        &mut self.data[index]
    }
}

#[cfg(test)]
mod test {
    use annotate_snippets::{
        display_list::{DisplayList, FormatOptions},
        snippet::{Annotation, AnnotationType, Slice, Snippet},
    };

    use super::BitValue;

    #[test]
    fn test_shr() {
        let a = -0b1i64;
        println!("{:b}", !(a << 4));
        let b = !(-0b1i64 << 4) as u64;
        let c = 0b11_1111u64;
        println!("{:b}", c & b);
    }

    #[test]
    fn test_op_code() {
        for i in 1..11 {
            let a: i8 = i;
            let b = !a + 1;
            println!("-{} => {:b}", a, b);
        }
    }

    #[test]
    fn test_saturating() {
        println!("{}", 0u8.saturating_sub(1))
    }

    #[test]
    fn test_to_bin() {
        let mut bv = BitValue::new(128);
        let a = i64::MAX.to_le_bytes();
        let mut rb = [0u8; 128 / 8];
        for (i, av) in a.iter().enumerate() {
            rb[i] = *av;
        }
        rb[9] = 0xff;
        let b = i128::from_le_bytes(rb);
        println!("{:x}", b);
        bv.parse_from(&format!("0b{:b}", b)).unwrap();
        assert_eq!(format!("{:0>128b}", b), format!("{}", bv.to_bin()));
    }

    #[test]
    fn test_to_bin_neg() {
        let mut bv = BitValue::new(128);
        let a = i64::MAX.to_le_bytes();
        let mut rb = [0u8; 128 / 8];
        for (i, av) in a.iter().enumerate() {
            rb[i] = *av;
        }
        rb[9] = 0xff;
        let b = -i128::from_le_bytes(rb);
        println!("{:x}", b);
        bv.parse_from(&format!("0b{:b}", b)).unwrap();
        println!("{:0>128b}", b);
        println!("{}", bv.to_bin());
        assert_eq!(format!("{:0>128b}", b), format!("{}", bv.to_bin()));
    }

    #[test]
    fn test_to_oct() {
        let mut bv = BitValue::new(128);
        let a = i64::MAX.to_le_bytes();
        let mut rb = [0u8; 128 / 8];
        for (i, av) in a.iter().enumerate() {
            rb[i] = *av;
        }
        rb[9] = 0xff;
        let b = i128::from_le_bytes(rb);
        println!("{:o}", b);

        match bv.parse_from(&format!("0o{:o}", b)) {
            Ok(_) => {}
            Err(s) => {
                // println!("{}", DisplayList::from(s));
                panic!("{}", DisplayList::from(s));
            }
        };
        assert_eq!(format!("{:0>44o}", b), format!("{}", bv.to_oct()));
    }

    #[test]
    fn test_to_hex() {
        let mut bv = BitValue::new(128);
        let a = i64::MAX.to_le_bytes();
        let mut rb = [0u8; 128 / 8];
        for (i, av) in a.iter().enumerate() {
            rb[i] = *av;
        }
        rb[9] = 0xff;
        let b = i128::from_le_bytes(rb);
        println!("{:o}", b);
        bv.parse_from(&format!("0x{:x}", b)).unwrap();
        assert_eq!(format!("{:0>32x}", b), format!("{}", bv.to_hex()));
    }

    #[test]
    fn test_to_dec() {
        let mut bv = BitValue::new(128);
        let a = i64::MAX.to_le_bytes();
        let mut rb = [0u8; 128 / 8];
        for (i, av) in a.iter().enumerate() {
            rb[i] = *av;
        }
        rb[9] = 0xff;
        let b = i128::from_le_bytes(rb);
        println!("{:o}", b);
        bv.parse_from(&format!("{}", b)).unwrap();
        assert_eq!(format!("{:0>40}", b), format!("{}", bv.to_dec()));
    }
}
