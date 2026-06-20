use crate::hash_table::control::group::{BitMaskWord, NonZeroBitMaskWord};

#[derive(Clone, Copy)]
pub(in crate::hash_table) struct BitMask(pub(in crate::hash_table) BitMaskWord);

impl BitMask {
    #[inline]
    #[must_use]
    fn remove_lowest_bit(self) -> Self {
        todo!()
    }

    #[inline]
    pub(in crate::hash_table) fn any_bit_set(self) -> bool {
        todo!()
    }

    #[inline]
    pub(in crate::hash_table) fn lowest_set_bit(self) -> Option<usize> {
        todo!()
    }

    #[inline]
    pub(in crate::hash_table) fn trailing_zeros(self) -> usize {
        todo!()
    }

    #[inline]
    pub(in crate::hash_table) fn nonzero_trailing_zeros(nonzero: NonZeroBitMaskWord) -> usize {
        todo!()
    }

    #[inline]
    pub(in crate::hash_table) fn leading_zeros(self) -> usize {
        todo!()
    }
}
impl IntoIterator for BitMask {
    type Item = usize;

    type IntoIter = BitMaskIter;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub(in crate::hash_table) struct BitMaskIter(pub(in crate::hash_table) BitMask);

impl Iterator for BitMaskIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
