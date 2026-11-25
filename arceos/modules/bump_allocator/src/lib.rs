#![cfg_attr(not(test), no_std)]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Aligns `addr` upwards to `align`, which must be a power of two.
const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Aligns `addr` downwards to `align`, which must be a power of two.
const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    /// Left boundary of the free area, i.e. the end of the byte area.
    b_pos: usize,
    /// Number of live byte allocations.
    b_count: usize,
    /// Left boundary of the page area.
    p_pos: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            b_count: 0,
            p_pos: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.b_count = 0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // The double-end layout can only describe one contiguous region.
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        // A zero-sized layout still gets a unique address, so bump by one.
        let size = layout.size().max(1);
        let alloc_start = align_up(self.b_pos, layout.align());
        let alloc_end = alloc_start
            .checked_add(size)
            .ok_or(AllocError::InvalidParam)?;

        if alloc_end > self.p_pos {
            return Err(AllocError::NoMemory); // would run into the page area
        }
        let ptr = NonNull::new(alloc_start as *mut u8).ok_or(AllocError::InvalidParam)?;

        self.b_pos = alloc_end;
        self.b_count += 1;
        Ok(ptr)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        // A bump allocator cannot reclaim a single region in the middle.
        // Only when the last live allocation is released can the entire byte
        // area be reset.
        self.b_count = self.b_count.saturating_sub(1);
        if self.b_count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos)
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        // Same contract as `BitmapPageAllocator`: the alignment is expressed in
        // bytes and must be a power of two no smaller than one page.
        if num_pages == 0 || align_pow2 == 0 || align_pow2 % SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        if !(align_pow2 / SIZE).is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        let size = num_pages
            .checked_mul(SIZE)
            .ok_or(AllocError::InvalidParam)?;
        let tail = self.p_pos.checked_sub(size).ok_or(AllocError::NoMemory)?;
        let alloc_start = align_down(tail, align_pow2);

        if alloc_start < self.b_pos {
            return Err(AllocError::NoMemory); // would collide with the byte area
        }

        self.p_pos = alloc_start;
        Ok(alloc_start)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Pages are allocated backwards and are never freed.
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos) / SIZE
    }
}
