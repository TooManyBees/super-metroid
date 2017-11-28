use util::snespc2;
use std::ops::{Add, Index, RangeFrom};

// FIXME: when `const fn` feature lands,
// remove `pub` from the element
pub struct Rom<'a>(pub &'a [u8]);

impl<'a> Rom<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Rom(slice)
    }

    pub fn read(&self, addr: PcAddress, len: usize) -> &'a [u8] {
        &self.0[addr.0 .. addr.0 + len]
    }
}

impl<'a> Index<PcAddress> for Rom<'a> {
    type Output = u8;
    fn index(&self, index: PcAddress) -> &Self::Output {
        &self.0[index.0]
    }
}

impl<'a> Index<RangeFrom<PcAddress>> for Rom<'a> {
    type Output = [u8];
    fn index(&self, index: RangeFrom<PcAddress>) -> &Self::Output {
        &self.0[index.start.0..]
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SnesAddress(pub u32);

impl SnesAddress {
    pub fn to_pc(&self) -> PcAddress {
        snespc2(self.0)
    }
}

impl Add<usize> for SnesAddress {
    type Output = PcAddress;
    fn add(self, rhs: usize) -> Self::Output {
        snespc2(self.0) + rhs
    }
}
