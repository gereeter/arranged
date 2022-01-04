#![feature(allocator_api, alloc_layout_extra, dropck_eyepatch)]
#![no_std]

pub use layouts::{Flat, Parallel, Slice, Strided};
#[cfg(feature="bitvec")]
pub use layouts::PackedBits;
pub use reference::{Ref, RefMut};

pub mod layouts;
pub mod reference;
