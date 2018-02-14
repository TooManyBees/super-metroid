#![allow(non_upper_case_globals)]
use core::cmp::{PartialEq, Eq};
use core::{fmt, ops};
use core::option::Option;

// bitflags!{
//     pub struct ControllerInput: u16 {
//         const DiagonalUp   = 0x0010;
//         const DiagonalDown = 0x0020;
//         const Shoot        = 0x0040;
//         const Jump         = 0x0080;
//         const Right        = 0x0100;
//         const Left         = 0x0200;
//         const Down         = 0x0400;
//         const Up           = 0x0800;
//         const Start        = 0x1000;
//         const Select       = 0x2000;
//         const Cancel       = 0x4000;
//         const Run          = 0x8000;
//     }
// }

#[derive(Copy, Clone)]
pub struct ControllerInput {
    pub bits: u16,
}

impl PartialEq for ControllerInput {
    #[inline]
    fn eq(&self, other: &ControllerInput) -> bool {
        self.bits == other.bits
    }

    #[inline]
    fn ne(&self, other: &ControllerInput) -> bool {
        self.bits != other.bits
    }
}

impl Eq for ControllerInput {}

impl fmt::Debug for ControllerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[allow(non_snake_case)]
        trait BitFlags {
            #[inline] fn DiagonalUp(&self) -> bool { false }
            #[inline] fn DiagonalDown(&self) -> bool { false }
            #[inline] fn Shoot(&self) -> bool { false }
            #[inline] fn Jump(&self) -> bool { false }
            #[inline] fn Right(&self) -> bool { false }
            #[inline] fn Left(&self) -> bool { false }
            #[inline] fn Down(&self) -> bool { false }
            #[inline] fn Up(&self) -> bool { false }
            #[inline] fn Start(&self) -> bool { false }
            #[inline] fn Select(&self) -> bool { false }
            #[inline] fn Cancel(&self) -> bool { false }
            #[inline] fn Run(&self) -> bool { false }
        }
        impl BitFlags for ControllerInput {
            #[inline]
            fn DiagonalUp(&self) -> bool {
                self.bits & Self::DiagonalUp.bits == Self::DiagonalUp.bits
            }
            #[inline]
            fn DiagonalDown(&self) -> bool {
                self.bits & Self::DiagonalDown.bits == Self::DiagonalDown.bits
            }
            #[inline]
            fn Shoot(&self) -> bool {
                self.bits & Self::Shoot.bits == Self::Shoot.bits
            }
            #[inline]
            fn Jump(&self) -> bool {
                self.bits & Self::Jump.bits == Self::Jump.bits
            }
            #[inline]
            fn Right(&self) -> bool {
                self.bits & Self::Right.bits == Self::Right.bits
            }
            #[inline]
            fn Left(&self) -> bool {
                self.bits & Self::Left.bits == Self::Left.bits
            }
            #[inline]
            fn Down(&self) -> bool {
                self.bits & Self::Down.bits == Self::Down.bits
            }
            #[inline]
            fn Up(&self) -> bool {
                self.bits & Self::Up.bits == Self::Up.bits
            }
            #[inline]
            fn Start(&self) -> bool {
                self.bits & Self::Start.bits == Self::Start.bits
            }
            #[inline]
            fn Select(&self) -> bool {
                self.bits & Self::Select.bits == Self::Select.bits
            }
            #[inline]
            fn Cancel(&self) -> bool {
                self.bits & Self::Cancel.bits == Self::Cancel.bits
            }
            #[inline]
            fn Run(&self) -> bool {
                self.bits & Self::Run.bits == Self::Run.bits
            }
        }
        let mut first = true;
        if <ControllerInput as BitFlags>::DiagonalUp(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("DiagonalUp")?;
        }
        if <ControllerInput as BitFlags>::DiagonalDown(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("DiagonalDown")?;
        }
        if <ControllerInput as BitFlags>::Shoot(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Shoot")?;
        }
        if <ControllerInput as BitFlags>::Jump(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Jump")?;
        }
        if <ControllerInput as BitFlags>::Right(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Right")?;
        }
        if <ControllerInput as BitFlags>::Left(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Left")?;
        }
        if <ControllerInput as BitFlags>::Down(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Down")?;
        }
        if <ControllerInput as BitFlags>::Up(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Up")?;
        }
        if <ControllerInput as BitFlags>::Start(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Start")?;
        }
        if <ControllerInput as BitFlags>::Select(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Select")?;
        }
        if <ControllerInput as BitFlags>::Cancel(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Cancel")?;
        }
        if <ControllerInput as BitFlags>::Run(self) {
            if !first { f.write_str(" | ")?; }
            first = false;
            f.write_str("Run")?;
        }
        if first {
            f.write_str("(empty)")?;
        }
        Ok(())
    }
}

impl fmt::Binary for ControllerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.bits, f)
    }
}
impl fmt::Octal for ControllerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.bits, f)
    }
}
impl fmt::LowerHex for ControllerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.bits, f)
    }
}
impl fmt::UpperHex for ControllerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.bits, f)
    }
}

impl ControllerInput {
    pub const DiagonalUp: ControllerInput = ControllerInput { bits: 16 };
    pub const DiagonalDown: ControllerInput = ControllerInput { bits: 32 };
    pub const Shoot: ControllerInput = ControllerInput { bits: 64 };
    pub const Jump: ControllerInput = ControllerInput { bits: 128 };
    pub const Right: ControllerInput = ControllerInput { bits: 256 };
    pub const Left: ControllerInput = ControllerInput { bits: 512 };
    pub const Down: ControllerInput = ControllerInput { bits: 1024 };
    pub const Up: ControllerInput = ControllerInput { bits: 2048 };
    pub const Start: ControllerInput = ControllerInput { bits: 4096 };
    pub const Select: ControllerInput = ControllerInput { bits: 8192 };
    pub const Cancel: ControllerInput = ControllerInput { bits: 16384 };
    pub const Run: ControllerInput = ControllerInput { bits: 32768 };

    /// Returns an empty set of flags.
    #[inline]
    pub fn empty() -> ControllerInput { ControllerInput { bits: 0 } }
    
    /// Returns the set containing all flags.
    #[inline]
    pub fn all() -> ControllerInput {
        #[allow(non_snake_case)]
        trait BitFlags {
            #[inline] fn DiagonalUp() -> u16 { 0 }
            #[inline] fn DiagonalDown() -> u16 { 0 }
            #[inline] fn Shoot() -> u16 { 0 }
            #[inline] fn Jump() -> u16 { 0 }
            #[inline] fn Right() -> u16 { 0 }
            #[inline] fn Left() -> u16 { 0 }
            #[inline] fn Down() -> u16 { 0 }
            #[inline] fn Up() -> u16 { 0 }
            #[inline] fn Start() -> u16 { 0 }
            #[inline] fn Select() -> u16 { 0 }
            #[inline] fn Cancel() -> u16 { 0 }
            #[inline] fn Run() -> u16 { 0 }
        }
        impl BitFlags for ControllerInput {
            #[inline] fn DiagonalUp() -> u16 { Self::DiagonalUp.bits }
            #[inline] fn DiagonalDown() -> u16 { Self::DiagonalDown.bits }
            #[inline] fn Shoot() -> u16 { Self::Shoot.bits }
            #[inline] fn Jump() -> u16 { Self::Jump.bits }
            #[inline] fn Right() -> u16 { Self::Right.bits }
            #[inline] fn Left() -> u16 { Self::Left.bits }
            #[inline] fn Down() -> u16 { Self::Down.bits }
            #[inline] fn Up() -> u16 { Self::Up.bits }
            #[inline] fn Start() -> u16 { Self::Start.bits }
            #[inline] fn Select() -> u16 { Self::Select.bits }
            #[inline] fn Cancel() -> u16 { Self::Cancel.bits }
            #[inline] fn Run() -> u16 { Self::Run.bits }
        }
        ControllerInput {
            bits:
                <ControllerInput as BitFlags>::DiagonalUp() |
                <ControllerInput as BitFlags>::DiagonalDown() |
                <ControllerInput as BitFlags>::Shoot() |
                <ControllerInput as BitFlags>::Jump() |
                <ControllerInput as BitFlags>::Right() |
                <ControllerInput as BitFlags>::Left() |
                <ControllerInput as BitFlags>::Down() |
                <ControllerInput as BitFlags>::Up() |
                <ControllerInput as BitFlags>::Start() |
                <ControllerInput as BitFlags>::Select() |
                <ControllerInput as BitFlags>::Cancel() |
                <ControllerInput as BitFlags>::Run()
        }
    }

    /// Returns the raw value of the flags currently stored.
    #[inline]
    pub fn bits(&self) -> u16 { self.bits }

    /// Convert from underlying bit representation, unless that
    /// representation contains bits that do not correspond to a flag.
    #[inline]
    pub fn from_bits(bits: u16) -> Option<ControllerInput> {
        if (bits & !ControllerInput::all().bits()) == 0 {
            Option::Some(ControllerInput { bits: bits })
        } else { Option::None }
    }

    /// Convert from underlying bit representation, dropping any bits
    /// that do not correspond to flags.
    #[inline]
    pub fn from_bits_truncate(bits: u16) -> ControllerInput {
        ControllerInput { bits: bits } & ControllerInput::all()
    }

    /// Returns `true` if no flags are currently stored.
    #[inline]
    pub fn is_empty(&self) -> bool { *self == ControllerInput::empty() }

    /// Returns `true` if all flags are currently set.
    #[inline]
    pub fn is_all(&self) -> bool { *self == ControllerInput::all() }

    /// Returns `true` if there are flags common to both `self` and `other`.
    #[inline]
    pub fn intersects(&self, other: ControllerInput) -> bool {
        !(*self & other).is_empty()
    }

    /// Returns `true` all of the flags in `other` are contained within `self`.
    #[inline]
    pub fn contains(&self, other: ControllerInput) -> bool {
        (*self & other) == other
    }

    /// Inserts the specified flags in-place.
    #[inline]
    pub fn insert(&mut self, other: ControllerInput) {
        self.bits |= other.bits;
    }

    /// Removes the specified flags in-place.
    #[inline]
    pub fn remove(&mut self, other: ControllerInput) {
        self.bits &= !other.bits;
    }

    /// Toggles the specified flags in-place.
    #[inline]
    pub fn toggle(&mut self, other: ControllerInput) {
        self.bits ^= other.bits;
    }

    /// Inserts or removes the specified flags depending on the passed value.
    #[inline]
    pub fn set(&mut self, other: ControllerInput, value: bool) {
        if value { self.insert(other); } else { self.remove(other); }
    }
}

impl ops::BitOr for ControllerInput {
    type Output = ControllerInput;
    /// Returns the union of the two sets of flags.
    #[inline]
    fn bitor(self, other: ControllerInput) -> ControllerInput {
        ControllerInput { bits: self.bits | other.bits}
    }
}

impl ops::BitOrAssign for ControllerInput {
    /// Adds the set of flags.
    #[inline]
    fn bitor_assign(&mut self, other: ControllerInput) {
        self.bits |= other.bits;
    }
}

impl ops::BitXor for ControllerInput {
    type Output = ControllerInput;
    /// Returns the left flags, but with all the right flags toggled.
    #[inline]
    fn bitxor(self, other: ControllerInput) -> ControllerInput {
        ControllerInput { bits: self.bits ^ other.bits}
    }
}

impl ops::BitXorAssign for ControllerInput {
    /// Toggles the set of flags.
    #[inline]
    fn bitxor_assign(&mut self, other: ControllerInput) {
        self.bits ^= other.bits;
    }
}

impl ops::BitAnd for ControllerInput {
    type Output = ControllerInput;
    /// Returns the intersection between the two sets of flags.
    #[inline]
    fn bitand(self, other: ControllerInput) -> ControllerInput {
        ControllerInput { bits: self.bits & other.bits}
    }
}

impl ops::BitAndAssign for ControllerInput {
    /// Disables all flags disabled in the set.
    #[inline]
    fn bitand_assign(&mut self, other: ControllerInput) {
        self.bits &= other.bits;
    }
}
impl ops::Sub for ControllerInput {
    type Output = ControllerInput;
    /// Returns the set difference of the two sets of flags.
    #[inline]
    fn sub(self, other: ControllerInput) -> ControllerInput {
        ControllerInput { bits: self.bits & !other.bits }
    }
}

impl ops::SubAssign for ControllerInput {
    /// Disables all flags enabled in the set.
    #[inline]
    fn sub_assign(&mut self, other: ControllerInput) {
        self.bits &= !other.bits;
    }
}

impl ops::Not for ControllerInput {
    type Output = ControllerInput;
    /// Returns the complement of this set of flags.
    #[inline]
    fn not(self) -> ControllerInput {
        ControllerInput { bits: !self.bits } & ControllerInput::all()
    }
}
