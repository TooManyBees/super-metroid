use std::ops::{Add, Index, Range, RangeFrom, RangeTo};
use std::fmt;

// #[inline(always)]
// pub fn snespc(bank: u8, addr: u16) -> PcAddress {
//     PcAddress(
//         (((bank & 127) as usize) << 15) + (addr as usize) - 512 - 32256
//     )
// }

#[inline(always)]
pub fn snespc(addr: u32) -> usize {
    (((addr & 0x7F0000) >> 1) + (addr & 0xFFFF)) as usize - 512 - 32256
}

// FIXME: when `const fn` feature lands, remove `pub` from
// the element and use a `::new` function instead
pub struct Rom<'a>(pub &'a [u8]);

impl<'a> Rom<'a> {
    pub fn read(&self, addr: PcAddress, len: usize) -> &'a [u8] {
        &self.0[addr.0 .. addr.0 + len]
    }

    pub fn read_string(&self, addr: PcAddress, max_len: usize) -> Option<String> {
        let mut v = Vec::new();
        for c in self.0[addr.0.. addr.0 + max_len].iter().take_while(|c| **c != 0x20 && **c != 0x00) {
            v.push(*c);
        }
        String::from_utf8(v).ok()
    }
}

impl<'a> Index<PcAddress> for Rom<'a> {
    type Output = u8;
    fn index(&self, index: PcAddress) -> &Self::Output {
        &self.0[index.0]
    }
}

impl <'a> Index<Range<PcAddress>> for Rom<'a> {
    type Output = [u8];
    fn index(&self, index: Range<PcAddress>) -> &Self::Output {
        &self.0[index.start.0..index.end.0]
    }
}

impl<'a> Index<RangeFrom<PcAddress>> for Rom<'a> {
    type Output = [u8];
    fn index(&self, index: RangeFrom<PcAddress>) -> &Self::Output {
        &self.0[index.start.0..]
    }
}

impl<'a> Index<RangeTo<PcAddress>> for Rom<'a> {
    type Output = [u8];
    fn index(&self, index: RangeTo<PcAddress>) -> &Self::Output {
        &self.0[..index.end.0]
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PcAddress(pub usize);

impl Add<usize> for PcAddress {
    type Output = PcAddress;
    fn add(self, rhs: usize) -> Self::Output {
        PcAddress(self.0 + rhs)
    }
}

impl fmt::Debug for PcAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PcAddress({:06X})", self.0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SnesAddress(pub u32);

impl SnesAddress {
    pub fn to_pc(self) -> PcAddress {
        self.into()
    }
}

impl Add<SnesAddress> for SnesAddress {
    type Output = SnesAddress;
    fn add(self, rhs: SnesAddress) -> Self::Output {
        SnesAddress(self.0 + rhs.0)
    }
}

impl Add<u32> for SnesAddress {
    type Output = SnesAddress;
    fn add(self, rhs: u32) -> Self::Output {
        SnesAddress(self.0 + rhs)
    }
}

impl Into<PcAddress> for SnesAddress {
    fn into(self) -> PcAddress {
        PcAddress(snespc(self.0))
    }
}

impl fmt::Debug for SnesAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SnesAddress(${:06X})", self.0)
    }
}
