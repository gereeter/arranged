use core::alloc::Layout;
use core::ptr::NonNull;
use core::marker::PhantomData;

use layouts::ArrayLayout;

pub struct Extra<T> {
    _marker: PhantomData<T>
}

unsafe impl<T> ArrayLayout<()> for Extra<T> {
    type Ptr = NonNull<T>;
    type ArrayInfo = ();

    fn layout_array(_count: usize) -> (Layout, Self::ArrayInfo) {
        (Layout::new::<T>(), ())
    }

    unsafe fn from_flat_ptr(ptr: NonNull<u8>, _info: Self::ArrayInfo) -> Self::Ptr {
        ptr.cast()
    }

    unsafe fn initialize(_ptr: Self::Ptr, _count: usize) { }

    unsafe fn base_ptr(ptr: Self::Ptr, _info: Self::ArrayInfo) -> NonNull<u8> {
        ptr.cast()
    }

    fn dangling() -> Self::Ptr {
        NonNull::dangling()
    }

    unsafe fn offset(ptr: Self::Ptr, _offset: isize) -> Self::Ptr {
        ptr
    }

    unsafe fn same_ptr(_ptr1: Self::Ptr, _ptr2: Self::Ptr) -> bool {
        // Since the "array element" of this type is zero-sized, we are allowed
        // to spuriously return true.
        true
    }

    unsafe fn read(_ptr: Self::Ptr) -> () { }
    unsafe fn write(_ptr: Self::Ptr, _value: ()) { }
    unsafe fn drop_in_place(_ptr: Self::Ptr) { }
    unsafe fn copy_one_nonoverlapping(_src: Self::Ptr, _dest: Self::Ptr) { }
    unsafe fn swap_one_nonoverlapping(_ptr1: Self::Ptr, _ptr2: Self::Ptr) { }
    unsafe fn copy_leftwards(_src: Self::Ptr, _dest: Self::Ptr, _count: usize) { }
    unsafe fn copy_rightwards(_src: Self::Ptr, _dest: Self::Ptr, _count: usize) { }
    unsafe fn copy_nonoverlapping(_src: Self::Ptr, _dest: Self::Ptr, _count: usize) { }
    unsafe fn swap_nonoverlapping(_ptr1: Self::Ptr, _ptr2: Self::Ptr, _count: usize) { }
}
