use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ptr::NonNull;

use layouts::ArrayLayout;

const USIZE_BITS: usize = size_of::<usize>() * 8;

pub struct PackedBits<WordLayout> {
    _marker: PhantomData<WordLayout>
}

pub struct BitPtr<WordLayout: ArrayLayout<usize>> {
    word_ptr: WordLayout::Ptr,
    bit_index: u8
}

impl<WordLayout: ArrayLayout<usize>> Copy for BitPtr<WordLayout> { }
impl<WordLayout: ArrayLayout<usize>> Clone for BitPtr<WordLayout> {
    fn clone(&self) -> Self { *self }
}

unsafe impl<WordLayout: ArrayLayout<usize>> ArrayLayout<bool> for PackedBits<WordLayout> {
    type Ptr = BitPtr<WordLayout>;
    type ArrayInfo = WordLayout::ArrayInfo;

    fn layout_array(count: usize) -> (Layout, Self::ArrayInfo) {
        let word_count = (count + USIZE_BITS - 1) / USIZE_BITS;
        WordLayout::layout_array(word_count)
    }

    unsafe fn from_flat_ptr(ptr: NonNull<u8>, info: Self::ArrayInfo) -> Self::Ptr {
        BitPtr {
            word_ptr: WordLayout::from_flat_ptr(ptr, info),
            bit_index: 0
        }
    }

    unsafe fn initialize(ptr: Self::Ptr, count: usize) {
        debug_assert!(ptr.bit_index == 0);
        WordLayout::initialize(ptr.word_ptr, count);
        // Filling the array with zeros is unnecessary, since we should only
        // read a bit after we have written this bit. This even is sound in
        // LLVM's current model where uninitialized data has `undef`, an arbitrary
        // fluctuating value. However, drafts for Rust's memory model and
        // proposals for LLVM make uninitialize memory non-bitwise poison,
        // which would make this unsound. Since Rust does not provide a `freeze`
        // intrinsic to safely deal with this yet, we hide using uninitialized
        // memory behind a feature flag.
        #[cfg(not(feature = "uninit_packedbits"))]
        {
            let word_count = (count + USIZE_BITS - 1) / USIZE_BITS;
            let word_end = WordLayout::offset(ptr.word_ptr, word_count as isize);
            let mut cur_word = ptr.word_ptr;
            while !WordLayout::same_ptr(cur_word, word_end) {
                WordLayout::write(cur_word, 0);
                cur_word = WordLayout::offset(cur_word, 1);
            }
        }
    }

    unsafe fn base_ptr(ptr: Self::Ptr, info: Self::ArrayInfo) -> NonNull<u8> {
        WordLayout::base_ptr(ptr.word_ptr, info)
    }

    fn dangling() -> Self::Ptr {
        BitPtr {
            word_ptr: WordLayout::dangling(),
            bit_index: 0
        }
    }

    unsafe fn offset(ptr: Self::Ptr, offset: isize) -> Self::Ptr {
        let offset_from_word = offset + ptr.bit_index as isize;
        let bit_index = (offset_from_word as usize) % USIZE_BITS;
        BitPtr {
            word_ptr: WordLayout::offset(ptr.word_ptr, (offset_from_word - bit_index as isize) / USIZE_BITS as isize),
            bit_index: bit_index as u8
        }
    }

    unsafe fn same_ptr(ptr1: Self::Ptr, ptr2: Self::Ptr) -> bool {
        WordLayout::same_ptr(ptr1.word_ptr, ptr2.word_ptr) && ptr1.bit_index == ptr2.bit_index
    }

    unsafe fn read(ptr: Self::Ptr) -> bool {
        ((WordLayout::read(ptr.word_ptr) >> ptr.bit_index) & 1) != 0
    }

    unsafe fn write(ptr: Self::Ptr, value: bool) {
        // FIXME: This reads uninitialized data!
        let old_word = WordLayout::read(ptr.word_ptr);
        let new_word = if value { // TODO: Non-branching
            old_word | (1 << ptr.bit_index)
        } else {
            old_word & !(1 << ptr.bit_index)
        };
        WordLayout::write(ptr.word_ptr, new_word);
    }

    unsafe fn drop_in_place(_ptr: Self::Ptr) { }

    // TODO: copy_nonoverlapping could be a lot more efficient
}

