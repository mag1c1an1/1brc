use std::ptr::NonNull;

pub(in crate::hash_table) use allocator_api2::alloc::{Allocator, Global};
pub(in crate::hash_table) fn do_alloc<A: Allocator>(
    allocator: &A,
    layout: core::alloc::Layout,
) -> core::result::Result<NonNull<[u8]>, ()> {
    match allocator.allocate(layout) {
        Ok(ptr) => Ok(ptr),
        Err(_) => Err(()),
    }
}
