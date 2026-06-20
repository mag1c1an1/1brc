use core::marker::PhantomData;
use core::ptr::NonNull;

use super::alloc::{Allocator, Global};

struct Bucket<T> {
    ptr: NonNull<T>,
}

struct RawTable<T, A: Allocator = Global> {
    table: RawTableInner,
    allocator: A,
    marker: PhantomData<T>,
}

impl<T, A: Allocator> RawTable<T, A> {
    /// Searches for an element in the table
    pub(in crate::hash_table) fn find(
        &self,
        hash: u64,
        mut eq: impl FnMut(&T) -> bool,
    ) -> Option<Bucket<T>> {
        todo!()
    }
}

struct RawTableInner {
    ctrl: NonNull<u8>,
}

impl RawTableInner {
    unsafe fn find_inner(&self, hash: u64, mut eq: &mut dyn FnMut(usize) -> bool) -> Option<usize> {
        todo!()
    }
}
