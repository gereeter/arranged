use core::alloc::Layout;
use core::ptr::{self, NonNull};

use layouts::ArrayLayout;

pub struct Flat {
    _priv: ()
}

unsafe impl<T> ArrayLayout<T> for Flat {
    type Ptr = NonNull<T>;
    type ArrayInfo = ();

    fn layout_array(count: usize) -> (Layout, ()) {
        (Layout::array::<T>(count).expect("Overflow in calculating array layout"), ())
    }

    unsafe fn from_flat_ptr(ptr: NonNull<u8>, _info: ()) -> NonNull<T> {
        ptr.cast()
    }

    unsafe fn initialize(_ptr: NonNull<T>, _count: usize) { }

    unsafe fn base_ptr(ptr: NonNull<T>, _info: ()) -> NonNull<u8> {
        ptr.cast()
    }

    fn dangling() -> NonNull<T> {
        NonNull::dangling()
    }

    unsafe fn offset(ptr: NonNull<T>, offset: isize) -> NonNull<T> {
        NonNull::new_unchecked(ptr.as_ptr().offset(offset))
    }

    unsafe fn same_ptr(ptr1: NonNull<T>, ptr2: NonNull<T>) -> bool {
        ptr1 == ptr2
    }

    unsafe fn read(ptr: NonNull<T>) -> T {
        ptr::read(ptr.as_ptr())
    }

    unsafe fn write(ptr: NonNull<T>, value: T) {
        ptr::write(ptr.as_ptr(), value);
    }

    unsafe fn drop_in_place(ptr: NonNull<T>) {
        ptr::drop_in_place(ptr.as_ptr());
    }

    unsafe fn copy_one_nonoverlapping(src: NonNull<T>, dest: NonNull<T>) {
        ptr::copy_nonoverlapping(src.as_ptr(), dest.as_ptr(), 1);
    }

    unsafe fn swap_one_nonoverlapping(ptr1: NonNull<T>, ptr2: NonNull<T>) {
        ptr::swap_nonoverlapping(ptr1.as_ptr(), ptr2.as_ptr(), 1);
    }

    unsafe fn copy_leftwards(src: NonNull<T>, dest: NonNull<T>, count: usize) {
        ptr::copy(src.as_ptr(), dest.as_ptr(), count);
    }

    unsafe fn copy_rightwards(src: NonNull<T>, dest: NonNull<T>, count: usize) {
        ptr::copy(src.as_ptr(), dest.as_ptr(), count);
    }

    unsafe fn copy_nonoverlapping(src: NonNull<T>, dest: NonNull<T>, count: usize) {
        ptr::copy_nonoverlapping(src.as_ptr(), dest.as_ptr(), count);
    }

    unsafe fn swap_nonoverlapping(ptr1: NonNull<T>, ptr2: NonNull<T>, count: usize) {
        ptr::swap_nonoverlapping(ptr1.as_ptr(), ptr2.as_ptr(), count);
    }
}
