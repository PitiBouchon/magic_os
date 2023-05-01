use crate::kalloc::{page_round_down, page_round_up};
use bit_field::BitField;

pub const MAX_VIRTUAL_ADDR: u64 = 1 << (9 + 9 + 9 + 12 - 1);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct VirtualAddr(u64);

#[derive(Debug)]
pub(super) struct VirtualPageNumber(pub(super) u16);

#[derive(Debug)]
pub struct PageOffset(pub u16);

impl VirtualAddr {
    pub const fn new(val: u64) -> Self {
        assert!(val <= MAX_VIRTUAL_ADDR);
        Self(val)
    }

    pub fn is_align(&self, align: u64) -> bool {
        self.0 % align == 0
    }

    pub const fn add_offset(self, offset: u64) -> Self {
        assert!(self.0 + offset <= MAX_VIRTUAL_ADDR);
        Self(self.0 + offset)
    }

    pub const fn sub_offset(self, offset: u64) -> Self {
        assert!(offset <= self.0);
        Self(self.0 - offset)
    }

    pub fn get(&self) -> &u64 {
        &self.0
    }

    pub fn page_round_down(self) -> Self {
        VirtualAddr(page_round_down(self.0))
    }

    pub fn page_round_up(self) -> Self {
        VirtualAddr(page_round_up(self.0))
    }

    pub(super) fn virtual_page_numbers(&self) -> [VirtualPageNumber; 3] {
        [
            VirtualPageNumber(self.0.get_bits(12..21) as u16),
            VirtualPageNumber(self.0.get_bits(21..30) as u16),
            VirtualPageNumber(self.0.get_bits(30..39) as u16),
        ]
    }

    pub(super) fn page_offset(&self) -> PageOffset {
        PageOffset((self.0.get_bits(0..12)) as u16)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PhysicalAddr(pub(super) u64);

impl PhysicalAddr {
    pub fn new(val: u64) -> Self {
        const MAX_PHYSICAL_ADDR: u64 = 1 << (26 + 9 + 9 + 12 - 1);

        assert!(val < MAX_PHYSICAL_ADDR);
        Self(val)
    }
}

#[derive(Debug)]
pub struct Ppn(pub(super) u64);

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
