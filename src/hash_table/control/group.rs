use core::arch::x86_64 as x86;
use core::mem;
use core::num::NonZero;

use crate::hash_table::control::tag::Tag;

pub(in crate::hash_table) type BitMaskWord = u16;
pub(in crate::hash_table) type NonZeroBitMaskWord = NonZero<u16>;
pub(in crate::hash_table) const BITMASK_STRIDE: usize = 1;
pub(in crate::hash_table) const BITMASK_ITER_MASK: usize = !0;

// sse2
struct Group(x86::__m128i);

impl Group {
    // 16
    const WIDTH: usize = mem::size_of::<Self>();

    pub(in crate::hash_table) const fn static_empty() -> &'static [Tag; Group::WIDTH] {
        todo!()
    }
}
