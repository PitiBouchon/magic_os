// See : The RISCV Privileged Manual
#![allow(unused)]

use crate::kalloc::{page_round_down, page_round_up};
use crate::println;
use crate::vm::page_table::addr::{PageOffset, PhysicalAddr, Ppn};
use crate::vm::page_table::entry::perm::{
    PTE_BIT_EXECUTE, PTE_BIT_READ, PTE_BIT_VALID, PTE_BIT_WRITE,
};
use bit_field::BitField;
use core::ops::{BitAnd, BitOr, BitOrAssign};
use perm::PTEPermission;

pub mod perm;

#[derive(Debug, Clone)]
pub struct PageTableEntry(pub u64);

#[derive(Debug, Eq, PartialEq)]
pub(super) enum EntryKind {
    Leaf,
    Branch(PhysicalAddr),
    NotValid,
}

impl PageTableEntry {
    pub fn new(ppn: Ppn, rsw: u8, perm: PTEPermission) -> Self {
        let mut value = 0u64;
        value.set_bits(0..8, perm.0 as u64);
        value.set_bits(8..10, rsw as u64); // These are just 2 bits free of use for the supervisor
        value.set_bits(10..54, ppn.0);
        Self(value)
    }

    pub(crate) fn convert_to_physical_addr(&self, offset: &PageOffset) -> PhysicalAddr {
        let mut res = 0u64;
        res.set_bits(0..12, offset.0 as u64);
        res.set_bits(12..55, self.ppn().0);
        PhysicalAddr(res)
    }

    pub fn new_zero() -> Self {
        Self(0)
    }

    pub fn is_valid(&self) -> bool {
        self.0.get_bit(PTE_BIT_VALID)
    }

    pub fn is_read(&self) -> bool {
        self.0.get_bit(PTE_BIT_READ)
    }

    pub fn is_write(&self) -> bool {
        self.0.get_bit(PTE_BIT_WRITE)
    }

    pub fn is_execute(&self) -> bool {
        self.0.get_bit(PTE_BIT_EXECUTE)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn perm(&self) -> PTEPermission {
        PTEPermission(self.0.get_bits(0..8) as u8)
    }

    pub(super) fn kind(&self) -> EntryKind {
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
        self.convert_to_physical_addr(&PageOffset(0))
    }

    fn ppn(&self) -> Ppn {
        Ppn(self.0.get_bits(10..54))
    }
}
