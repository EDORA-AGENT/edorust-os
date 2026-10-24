// EDORA NOVA: read-only physical memory map inspection.
// This module does NOT allocate or modify physical memory.

use core::ptr::read_volatile;
use core::sync::atomic::{AtomicU64, Ordering};

/// Limine memory map response (ABI layout).
#[repr(C)]
pub struct MemoryMapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *mut *mut MemoryMapEntry,
}

/// Limine memory map entry (ABI layout).
#[repr(C)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub typ: u64,
}

/// Limine memory-map type for usable RAM.
const LIMINE_MEMMAP_USABLE: u64 = 0;

/// Total usable bytes reported by Limine.
/// This is informational only; no frames are allocated.
static USABLE_BYTES: AtomicU64 = AtomicU64::new(0);
static ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);

/// Read and validate Limine's memory map.
pub fn init(response_ptr: *mut MemoryMapResponse) -> bool {
    if response_ptr.is_null() {
        return false;
    }

    unsafe {
        let response = read_volatile(response_ptr);

        // Refuse unreasonable counts before walking the array.
        if response.entry_count > 4096 {
            return false;
        }

        if response.entry_count > 0 && response.entries.is_null() {
            return false;
        }

        let mut usable_bytes = 0u64;
        let mut i = 0u64;

        while i < response.entry_count {
            let entry_ptr = read_volatile(response.entries.add(i as usize));

            if entry_ptr.is_null() {
                return false;
            }

            let entry = read_volatile(entry_ptr);

            if entry.typ == LIMINE_MEMMAP_USABLE {
                usable_bytes = usable_bytes.saturating_add(entry.length);
            }

            i += 1;
        }

        ENTRY_COUNT.store(response.entry_count, Ordering::Relaxed);
        USABLE_BYTES.store(usable_bytes, Ordering::Relaxed);
    }

    true
}

/// Number of memory map entries observed.
pub fn entry_count() -> u64 {
    ENTRY_COUNT.load(Ordering::Relaxed)
}

/// Bytes Limine reports as usable RAM. Not an allocation guarantee.
pub fn usable_bytes() -> u64 {
    USABLE_BYTES.load(Ordering::Relaxed)
}