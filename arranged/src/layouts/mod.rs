pub mod bitvec;
pub mod extra;
pub mod flat;
pub mod parallel;
pub mod slice;
pub mod strided;

pub use self::extra::Extra;
pub use self::flat::Flat;
pub use self::parallel::Parallel;
pub use self::slice::Slice;
pub use self::strided::Strided;
pub use self::bitvec::PackedBits;

use core::alloc::Layout;
use core::ptr::NonNull;

pub unsafe trait ArrayLayout<T: ?Sized> {
    type Ptr: Copy;
    type ArrayInfo;

    fn layout_array(count: usize) -> (Layout, Self::ArrayInfo) where T: Sized;
    unsafe fn from_flat_ptr(ptr: NonNull<u8>, info: Self::ArrayInfo) -> Self::Ptr where T: Sized;
    unsafe fn initialize(ptr: Self::Ptr, count: usize) where T: Sized;
    unsafe fn base_ptr(ptr: Self::Ptr, info: Self::ArrayInfo) -> NonNull<u8> where T: Sized;

    fn dangling() -> Self::Ptr;
    unsafe fn offset(ptr: Self::Ptr, offset: isize) -> Self::Ptr where T: Sized;
    unsafe fn same_ptr(ptr1: Self::Ptr, ptr2: Self::Ptr) -> bool;

    unsafe fn read(ptr: Self::Ptr) -> T where T: Sized;
    unsafe fn write(ptr: Self::Ptr, value: T) where T: Sized;
    unsafe fn drop_in_place(ptr: Self::Ptr);

    unsafe fn copy_one_nonoverlapping(src: Self::Ptr, dest: Self::Ptr) where T: Sized {
        Self::write(dest, Self::read(src));
    }
    unsafe fn swap_one_nonoverlapping(ptr1: Self::Ptr, ptr2: Self::Ptr) where T: Sized {
        let temp = Self::read(ptr1);
        Self::copy_one_nonoverlapping(ptr2, ptr1);
        Self::write(ptr2, temp);
    }

    unsafe fn copy_leftwards(mut src: Self::Ptr, mut dest: Self::Ptr, count: usize) where T: Sized {
        let src_end = Self::offset(src, count as isize);
        while !Self::same_ptr(src, src_end) {
            Self::copy_one_nonoverlapping(src, dest);
            src = Self::offset(src, 1);
            dest = Self::offset(dest, 1);
        }
    }
    unsafe fn copy_rightwards(mut src: Self::Ptr, mut dest: Self::Ptr, count: usize) where T: Sized {
        let src_end = src;
        src = Self::offset(src, count as isize);
        dest = Self::offset(dest, count as isize);
        while !Self::same_ptr(src, src_end) {
            src = Self::offset(src, -1);
            dest = Self::offset(dest, -1);
            Self::copy_one_nonoverlapping(src, dest);
        }
    }
    unsafe fn copy_nonoverlapping(src: Self::Ptr, dest: Self::Ptr, count: usize) where T: Sized {
        Self::copy_leftwards(src, dest, count);
    }
    unsafe fn swap_nonoverlapping(mut ptr1: Self::Ptr, mut ptr2: Self::Ptr, count: usize) where T: Sized {
        let ptr1_end = Self::offset(ptr1, count as isize);
        while !Self::same_ptr(ptr1, ptr1_end) {
            Self::swap_one_nonoverlapping(ptr1, ptr2);
            ptr1 = Self::offset(ptr1, 1);
            ptr2 = Self::offset(ptr2, 1);
        }
    }
}
