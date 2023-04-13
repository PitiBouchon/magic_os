// See : The RISCV Privileged Manual

use bit_field::BitField;
use crate::kalloc::{page_round_down, page_round_up};
use crate::println;

#[derive(Debug, Eq, PartialEq)]
pub struct VirtualAddr(pub u64);

#[derive(Debug)]
pub struct VirtualPageNumber(pub u16);

#[derive(Debug)]
pub struct PageOffset(u16);

impl VirtualAddr {
    pub fn is_align(&self, align: u64) -> bool {
        self.0 % align == 0
    }

    pub fn offset_by(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }

    pub fn page_round_down(self) -> Self {
        VirtualAddr(page_round_down(self.0))
    }

    pub fn page_round_up(self) -> Self {
        VirtualAddr(page_round_up(self.0))
    }

    pub fn virtual_page_numbers(&self) -> [VirtualPageNumber; 3] {
        [
            VirtualPageNumber(self.0.get_bits(12..21) as u16),
            VirtualPageNumber(self.0.get_bits(21..30) as u16),
            VirtualPageNumber(self.0.get_bits(30..39) as u16),
        ]
    }

    pub fn page_offset(&self) -> PageOffset {
        PageOffset((self.0.get_bits(0..12)) as u16)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PhysicalAddr(pub u64);

#[derive(Debug)]
pub struct Ppn(u64);

impl Ppn {
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl PhysicalAddr {
    pub fn is_align(&self, align: u64) -> bool {
        self.0 % align == 0
    }

    pub fn page_round_down(self) -> Self {
        PhysicalAddr(page_round_down(self.0))
    }

    pub fn page_round_up(self) -> Self {
        PhysicalAddr(page_round_up(self.0))
    }

    pub fn page_offset(&self) -> PageOffset {
        PageOffset((self.0.get_bits(0..12)) as u16)
    }

    pub fn ppn(&self) -> Ppn {
        Ppn(self.0.get_bits(12..54))
    }
}

#[derive(Debug, Clone)]
pub struct PageTableEntry(u64);

#[derive(Debug)]
pub struct Permission(pub u8);

// TODO : Find a solution to this copy-paste

impl From<(PageTableEntry, PageOffset)> for PhysicalAddr {
    fn from((entry, offset): (PageTableEntry, PageOffset)) -> Self {
        let mut res = 0u64;
        res.set_bits(0..12, offset.0 as u64);
        res.set_bits(12..55, entry.ppn().0);
        PhysicalAddr(res)
    }
}

impl From<(&PageTableEntry, PageOffset)> for PhysicalAddr {
    fn from((entry, offset): (&PageTableEntry, PageOffset)) -> Self {
        let mut res = 0u64;
        res.set_bits(0..12, offset.0 as u64);
        res.set_bits(12..55, entry.ppn().0);
        PhysicalAddr(res)
    }
}

impl From<(&PageTableEntry, &PageOffset)> for PhysicalAddr {
    fn from((entry, offset): (&PageTableEntry, &PageOffset)) -> Self {
        let mut res = 0u64;
        res.set_bits(0..12, offset.0 as u64);
        res.set_bits(12..55, entry.ppn().0);
        PhysicalAddr(res)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EntryKind {
    Leaf,
    Branch(PhysicalAddr),
    NotValid
}

pub const PTE_VALID: u8 = 0b0000_0001;
pub const PTE_READ: u8 = 0b0000_0010;
pub const PTE_WRITE: u8 = 0b0000_0100;
pub const PTE_EXECUTE: u8 = 0b0000_1000;
// const PTE_USER: u8 = 0b0001_0000;
// const PTE_GLOBAL: u8 = 0b0010_0000;
// const PTE_ACCESS: u8 = 0b0100_0000;
// const PTE_DIRTY: u8 = 0b1000_0000;

impl PageTableEntry {
    pub fn new(ppn: Ppn, rsw: u8, perm: Permission) -> Self {
        let mut value = 0u64;
        value.set_bits(0..8, perm.0 as u64);
        value.set_bits(8..10, rsw as u64);
        value.set_bits(10..54, ppn.0);
        Self(value)
    }

    pub fn is_valid(&self) -> bool {
        self.0 & PTE_VALID as u64 != 0
    }

    pub fn is_read(&self) -> bool {
        self.0 & PTE_READ as u64 != 0
    }

    pub fn is_write(&self) -> bool {
        self.0 & PTE_WRITE as u64 != 0
    }

    pub fn is_execute(&self) -> bool {
        self.0 & PTE_EXECUTE as u64 != 0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn perm(&self) -> Permission {
        Permission(self.0.get_bits(0..8) as u8)
    }

    pub fn kind(&self) -> EntryKind {
        if (!self.is_valid()) || (!self.is_valid() && self.is_write()) {
            return EntryKind::NotValid;
        }
        if self.is_read() || self.is_execute() {
            return EntryKind::Leaf;
        }
        let pa = self.addr_zero_offset();
        EntryKind::Branch(pa)
    }

    fn addr_zero_offset(&self) -> PhysicalAddr {
        (self.clone(), PageOffset(0)).into()
    }

    fn ppn(&self) -> Ppn {
        Ppn(self.0.get_bits(10..54))
    }
}
