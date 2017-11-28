use std::ops::{Add, Index, RangeFrom};

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
        PcAddress(snespc(self.0))
    }
}

impl Into<PcAddress> for SnesAddress {
    fn into(self) -> PcAddress {
        PcAddress(snespc(self.0))
    }
}
