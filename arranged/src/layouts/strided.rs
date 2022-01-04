use core::alloc::Layout;
use core::mem::size_of;
use core::ptr::{self, NonNull};

use reference::{Ref, RefMut};
use layouts::ArrayLayout;
use layouts::flat::Flat;
use layouts::slice::{Slice, SlicePtr};

pub struct Strided {
    _priv: ()
}

pub struct StridedPtr<T> {
    ptr: NonNull<T>,
    stride: isize
}

impl<T> Copy for StridedPtr<T> { }
impl<T> Clone for StridedPtr<T> {
    fn clone(&self) -> Self { *self }
}

unsafe impl<T> ArrayLayout<T> for Strided {
    type Ptr = StridedPtr<T>;
    type ArrayInfo = ();

    fn layout_array(count: usize) -> (Layout, ()) {
        (Layout::array::<T>(count).expect("Overflow in calculating array layout"), ())
    }

    unsafe fn from_flat_ptr(ptr: NonNull<u8>, _info: ()) -> StridedPtr<T> {
        StridedPtr {
            ptr: ptr.cast(),
            stride: size_of::<T>() as isize
        }
    }

    unsafe fn initialize(_ptr: StridedPtr<T>, _count: usize) { }

    unsafe fn base_ptr(ptr: StridedPtr<T>, _info: ()) -> NonNull<u8> {
        ptr.ptr.cast()
    }

    fn dangling() -> StridedPtr<T> {
        StridedPtr {
            ptr: NonNull::dangling(),
            stride: 1
        }
    }

    unsafe fn offset(ptr: StridedPtr<T>, offset: isize) -> StridedPtr<T> {
        StridedPtr {
            ptr: NonNull::new_unchecked(ptr.ptr.cast::<u8>().as_ptr().offset(offset * ptr.stride)).cast(),
            stride: ptr.stride
        }
    }

    unsafe fn same_ptr(ptr1: StridedPtr<T>, ptr2: StridedPtr<T>) -> bool {
        ptr1.ptr == ptr2.ptr
    }

    unsafe fn read(ptr: StridedPtr<T>) -> T {
        ptr::read(ptr.ptr.as_ptr())
    }

    unsafe fn write(ptr: StridedPtr<T>, value: T) {
        ptr::write(ptr.ptr.as_ptr(), value);
    }

    unsafe fn drop_in_place(ptr: StridedPtr<T>) {
        ptr::drop_in_place(ptr.ptr.as_ptr());
    }

    unsafe fn copy_one_nonoverlapping(src: Self::Ptr, dest: Self::Ptr) {
        ptr::copy_nonoverlapping(src.ptr.as_ptr(), dest.ptr.as_ptr(), 1);
    }

    unsafe fn swap_one_nonoverlapping(ptr1: Self::Ptr, ptr2: Self::Ptr) {
        ptr::swap_nonoverlapping(ptr1.ptr.as_ptr(), ptr2.ptr.as_ptr(), 1);
    }
}

impl<'a, T> Ref<'a, [T], Slice<Flat>> {
    pub fn strided(self) -> Ref<'a, [T], Slice<Strided>> {
        unsafe {
            Ref::from_raw(SlicePtr::from_raw_parts(
                StridedPtr {
                    ptr: self.as_ptr(),
                    stride: size_of::<T>() as isize
                },
                self.len()
            ))
        }
    }
}

impl<'a, T> RefMut<'a, [T], Slice<Flat>> {
    pub fn strided(self) -> RefMut<'a, [T], Slice<Strided>> {
        unsafe {
            RefMut::from_raw(SlicePtr::from_raw_parts(
                StridedPtr {
                    ptr: self.as_ptr(),
                    stride: size_of::<T>() as isize
                },
                self.len()
            ))
        }
    }
}

macro_rules! unzip_impl {
    { $($T:ident $val:ident),+ } => {
        impl<'a, $($T),+> Ref<'a, [($($T),+)], Slice<Strided>> {
            pub fn unzip(self) -> ($(Ref<'a, [$T], Slice<Strided>>),+) {
                let ($($val),+) = unsafe {
                    let this = self.as_ptr().ptr;
                    let &($(ref $val),+) = this.as_ref();
                    ($($val as *const _ as *mut _),+)
                };
                unsafe { ($(
                    Ref::from_raw(SlicePtr::from_raw_parts(
                        StridedPtr {
                            ptr: NonNull::new_unchecked($val),
                            stride: self.as_ptr().stride
                        },
                        self.len()
                    ))
                ),+) }
            }
        }

        impl<'a, $($T),+> RefMut<'a, [($($T),+)], Slice<Strided>> {
            pub fn unzip(self) -> ($(RefMut<'a, [$T], Slice<Strided>>),+) {
                let ($($val),+) = unsafe {
                    let mut this = self.as_ptr().ptr;
                    let &mut ($(ref mut $val),+) = this.as_mut();
                    ($($val as *mut _),+)
                };
                unsafe { ($(
                    RefMut::from_raw(SlicePtr::from_raw_parts(
                        StridedPtr {
                            ptr: NonNull::new_unchecked($val),
                            stride: self.as_ptr().stride
                        },
                        self.len()
                    ))
                ),+) }
            }
        }
    };
}

unzip_impl!{ A a, B b }
unzip_impl!{ A a, B b, C c }
unzip_impl!{ A a, B b, C c, D d }
unzip_impl!{ A a, B b, C c, D d, E e }
unzip_impl!{ A a, B b, C c, D d, E e, F f }
unzip_impl!{ A a, B b, C c, D d, E e, F f, G g }
unzip_impl!{ A a, B b, C c, D d, E e, F f, G g, H h }
