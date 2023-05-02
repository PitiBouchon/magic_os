use bit_field::BitField;
use core::ops::BitOr;

pub const PTE_BIT_VALID: usize = 0;
pub const PTE_BIT_READ: usize = 1;
pub const PTE_BIT_WRITE: usize = 2;
pub const PTE_BIT_EXECUTE: usize = 3;
pub const PTE_BIT_USER: usize = 4;

#[derive(Debug, Copy, Clone)]
pub struct PTEPermission(pub u8);

impl PTEPermission {
    pub fn new() -> Self {
        Self(0)
    }

    // See section 4.3.1 Addressing and Memory Protection (RiscV privileged manual)
    pub fn valid() -> Self {
        let mut res = 0;
        res.set_bit(PTE_BIT_VALID, true);
        Self(res)
    }

    pub fn read() -> Self {
        let mut res = 0;
        res.set_bit(PTE_BIT_READ, true);
        Self(res)
    }

    pub fn write() -> Self {
        let mut res = 0;
        res.set_bit(PTE_BIT_WRITE, true);
        Self(res)
    }

    pub fn execute() -> Self {
        let mut res = 0;
        res.set_bit(PTE_BIT_EXECUTE, true);
        Self(res)
    }

    pub fn user() -> Self {
        let mut res = 0;
        res.set_bit(PTE_BIT_USER, true);
        Self(res)
    }

    // TODO : set_global (maybe set_dirty and set_access)
}

impl BitOr for PTEPermission {
    type Output = PTEPermission;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
