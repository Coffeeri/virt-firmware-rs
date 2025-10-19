extern crate alloc;
use alloc::alloc::{alloc_zeroed, Layout};
use core::ops::Range;
use log::{debug, error, trace, warn};
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::structures::paging::mapper::MappedFrame;
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};

use efiloader::memorytype::*;

use virtfw_libhw::x86cpuid::has_gb_pages;

const PGSIZE_4K: u64 = 4096;
const PGSIZE_2M: u64 = PGSIZE_4K * 512;
const PGSIZE_1G: u64 = PGSIZE_2M * 512;

const PGTABLE_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(PGSIZE_4K as usize, PGSIZE_4K as usize) };
const PGTABLE_FLAGS: PageTableFlags = PageTableFlags::empty()
    .union(PageTableFlags::PRESENT)
    .union(PageTableFlags::WRITABLE)
    .union(PageTableFlags::ACCESSED);
const PAGE_FLAGS: PageTableFlags = PageTableFlags::empty()
    .union(PageTableFlags::PRESENT)
    .union(PageTableFlags::WRITABLE)
    .union(PageTableFlags::ACCESSED)
    .union(PageTableFlags::USER_ACCESSIBLE)
    .union(PageTableFlags::DIRTY);
const HUGE_PAGE_FLAGS: PageTableFlags = PageTableFlags::empty()
    .union(PAGE_FLAGS)
    .union(PageTableFlags::HUGE_PAGE);

pub struct MemoryMapper {
    pg4: &'static mut PageTable,
    have_1g: bool,
}

impl MemoryMapper {
    pub fn new() -> Self {
        let (cr3, _flags) = Cr3::read();
        let pg4 = Self::pgtable(cr3);
        let have_1g = has_gb_pages();
        Self { pg4, have_1g }
    }

    fn pgtable(frame: PhysFrame) -> &'static mut PageTable {
        let phys = frame.start_address();
        let virt = VirtAddr::new(phys.as_u64());
        let ptr: *mut PageTable = virt.as_mut_ptr();
        unsafe { &mut *ptr }
    }

    fn lookup_virt(&self, virt: VirtAddr) -> Option<(&mut PageTableEntry, MappedFrame)> {
        let e4 = &self.pg4[virt.p4_index()];
        trace!("l4    - {:?} - {:?}", e4.addr(), e4.flags());
        if !e4.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let pg3 = Self::pgtable(e4.frame().unwrap());
        let e3 = &mut pg3[virt.p3_index()];
        trace!(" l3   - {:?} - {:?}", e3.addr(), e3.flags());
        if !e3.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        if e3.flags().contains(PageTableFlags::HUGE_PAGE) {
            // 1G page
            let frame = MappedFrame::Size1GiB(PhysFrame::from_start_address(e3.addr()).unwrap());
            return Some((e3, frame));
        }

        let pg2 = Self::pgtable(e3.frame().unwrap());
        let e2 = &mut pg2[virt.p2_index()];
        trace!("  l2  - {:?} - {:?}", e2.addr(), e2.flags());
        if !e2.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        if e2.flags().contains(PageTableFlags::HUGE_PAGE) {
            // 2M page
            let frame = MappedFrame::Size2MiB(PhysFrame::from_start_address(e2.addr()).unwrap());
            return Some((e2, frame));
        }

        let pg1 = Self::pgtable(e2.frame().unwrap());
        let e1 = &mut pg1[virt.p1_index()];
        trace!("   l1 - {:?} - {:?}", e1.addr(), e1.flags());
        if !e1.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        // 4k page
        let frame = MappedFrame::Size4KiB(PhysFrame::from_start_address(e1.addr()).unwrap());
        Some((e1, frame))
    }

    fn fill_pgtable(
        &self,
        entry: &mut PageTableEntry,
        flags: PageTableFlags,
        start: PhysAddr,
        step: u64,
    ) {
        let table: &mut PageTable = unsafe {
            let page = alloc_zeroed(PGTABLE_LAYOUT);
            &mut *(page as *mut PageTable)
        };
        let taddr = PhysAddr::new(table as *const _ as u64);
        debug!("pgtable: new at {taddr:?} for {start:?}");

        for (i, e) in table.iter_mut().enumerate() {
            let addr = start + step * i as u64;
            e.set_addr(addr, flags);
        }

        entry.set_addr(taddr, PGTABLE_FLAGS);
    }

    fn split_entry(&self, entry: &mut PageTableEntry, start: PhysAddr, step: u64) {
        let mut flags = entry.flags();
        if step == PGSIZE_4K {
            flags.remove(PageTableFlags::HUGE_PAGE)
        }

        self.fill_pgtable(entry, flags, start, step);
    }

    fn update_range(
        &self,
        range: &Range<usize>,
        set: PageTableFlags,
        clr: PageTableFlags,
    ) -> Result<(), &str> {
        debug!(
            "update: range=0x{:x}-0x{:x} set={set:?} clr={clr:?}",
            range.start, range.end
        );
        let mut addr = range.start as u64;
        loop {
            let entry_opt = self.lookup_virt(VirtAddr::new(addr));
            if entry_opt.is_none() {
                return Err("address not mapped");
            }
            let (entry, frame) = entry_opt.unwrap();
            trace!("update: frame={:?}", frame);

            let start = frame.start_address().as_u64();
            let end = start + frame.size();
            if (addr != start) || (end as usize > range.end) {
                self.split_entry(entry, frame.start_address(), frame.size() / 512);
                continue;
            }

            let mut newflags = entry.flags();
            newflags.insert(set);
            newflags.remove(clr);
            entry.set_flags(newflags);

            if end as usize == range.end {
                // all done
                break;
            }
            addr = end;
        }
        tlb::flush_all();
        Ok(())
    }

    pub fn map_range(&self, range: &Range<usize>) {
        let mut virt = VirtAddr::new(range.start as u64);
        if !virt.is_aligned(PGSIZE_1G) {
            error!("range not aligned (1G)");
            return;
        }
        loop {
            let e4 = &self.pg4[virt.p4_index()];
            if !e4.flags().contains(PageTableFlags::PRESENT) {
                // needs fixing to go beyond 512 GB of memory
                error!("l4 not present [TODO]");
                break;
            }

            let pg3 = Self::pgtable(e4.frame().unwrap());
            let e3 = &mut pg3[virt.p3_index()];
            if e3.flags().contains(PageTableFlags::PRESENT) {
                warn!("l3 already present");
            } else {
                let phys = PhysAddr::new(virt.as_u64());
                if self.have_1g {
                    debug!("mapping {phys:?} [1G entry]");
                    e3.set_addr(phys, HUGE_PAGE_FLAGS);
                } else {
                    debug!("mapping {phys:?} [pgtable with 2M entries]");
                    self.fill_pgtable(e3, HUGE_PAGE_FLAGS, phys, PGSIZE_2M);
                }
            }

            virt += PGSIZE_1G;
            if virt.as_u64() as usize >= range.end {
                break;
            }
        }
        tlb::flush_all();
    }

    fn efi_to_flags(efi_set: u64, efi_clr: u64) -> (PageTableFlags, PageTableFlags) {
        let mut set = PageTableFlags::empty();
        let mut clr = PageTableFlags::empty();

        if (efi_set & EFI_MEMORY_RO) != 0 {
            clr.insert(PageTableFlags::WRITABLE);
        }
        if (efi_clr & EFI_MEMORY_RO) != 0 {
            set.insert(PageTableFlags::WRITABLE);
        }

        if (efi_set & EFI_MEMORY_XP) != 0 {
            set.insert(PageTableFlags::NO_EXECUTE);
        }
        if (efi_clr & EFI_MEMORY_XP) != 0 {
            clr.insert(PageTableFlags::NO_EXECUTE);
        }

        if (efi_set & EFI_MEMORY_UC) != 0 {
            set.insert(PageTableFlags::NO_CACHE);
        }
        if (efi_clr & EFI_MEMORY_UC) != 0 {
            clr.insert(PageTableFlags::NO_CACHE);
        }

        (set, clr)
    }
}

impl Default for MemoryMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl efiloader::MemoryMapper for MemoryMapper {
    fn remap_range(&self, range: &Range<usize>, set: u64, clr: u64) -> Result<(), &str> {
        let (s, c) = Self::efi_to_flags(set, clr);
        self.update_range(range, s, c)
    }

    fn query_range(&self, range: &Range<usize>) -> Option<u64> {
        debug!("query: range=0x{:x}-0x{:x} [TODO]", range.start, range.end);
        None
    }
}
