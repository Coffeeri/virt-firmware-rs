use linked_list_allocator::LockedHeap;
use log::info;

#[macro_export]
macro_rules! symrange {
    ($start:ident, $end:ident) => {
        unsafe {
            extern "C" {
                static $start: u8;
                static $end: u8;
            }
            (&$start as *const _ as usize)..(&$end as *const _ as usize)
        }
    };
}

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn alloc_init() {
    let heap = symrange!(heap_start, heap_end);
    unsafe {
        ALLOCATOR
            .lock()
            .init(heap.start as *mut u8, heap.end - heap.start);
    }
    info!("allocator initialized with heap");
}
