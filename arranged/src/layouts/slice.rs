use core::alloc::Layout;
use core::marker::PhantomData;
use core::ptr::NonNull;

use reference::{Ref, RefMut};
use layouts::ArrayLayout;

pub struct Slice<Inner> {
    _marker: PhantomData<Inner>
}

pub struct SlicePtr<T, TLayout: ArrayLayout<T>> {
    base: TLayout::Ptr,
    count: usize,
    _marker: PhantomData<T>
}

impl<T, TLayout: ArrayLayout<T>> Copy for SlicePtr<T, TLayout> { }
impl<T, TLayout: ArrayLayout<T>> Clone for SlicePtr<T, TLayout> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, TLayout: ArrayLayout<T>> SlicePtr<T, TLayout> {
    pub fn from_raw_parts(ptr: TLayout::Ptr, count: usize) -> Self {
        SlicePtr {
            base: ptr,
            count: count,
            _marker: PhantomData
        }
    }

    pub fn as_ptr(&self) -> TLayout::Ptr {
        self.base
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

impl<'a, T, TLayout: ArrayLayout<T>> Ref<'a, [T], Slice<TLayout>> {
    pub fn as_ptr(&self) -> TLayout::Ptr {
        self.as_raw().as_ptr()
    }

    pub fn len(&self) -> usize {
        self.as_raw().len()
    }
}

impl<'a, T, TLayout: ArrayLayout<T>> RefMut<'a, [T], Slice<TLayout>> {
    pub fn as_ptr(&self) -> TLayout::Ptr {
        self.as_raw().as_ptr()
    }

    pub fn len(&self) -> usize {
        self.as_raw().len()
    }
}

unsafe impl<T, TLayout> ArrayLayout<[T]> for Slice<TLayout> where TLayout: ArrayLayout<T> {
    type Ptr = SlicePtr<T, TLayout>;
    type ArrayInfo = ();

    fn dangling() -> Self::Ptr {
        SlicePtr::from_raw_parts(TLayout::dangling(), 0)
    }

    unsafe fn same_ptr(ptr1: Self::Ptr, ptr2: Self::Ptr) -> bool {
        TLayout::same_ptr(ptr1.base, ptr2.base)
    }

    unsafe fn drop_in_place(ptr: Self::Ptr) {
        for index in 0..ptr.count {
            TLayout::drop_in_place(TLayout::offset(ptr.base, index as isize));
        }
    }

    // Hack around the fact that Rust still requires implementations with unsatisfiable constraints
    fn layout_array(_count: usize) -> (Layout, Self::ArrayInfo) where [T]: Sized { panic!("[T] is not sized") }
    unsafe fn from_flat_ptr(_ptr: NonNull<u8>, _info: Self::ArrayInfo) -> Self::Ptr where [T]: Sized { panic!("[T] is not sized") }
    unsafe fn initialize(_ptr: Self::Ptr, _count: usize) where [T]: Sized { panic!("[T] is not sized") }
    unsafe fn base_ptr(_ptr: Self::Ptr, _info: Self::ArrayInfo) -> NonNull<u8> where [T]: Sized { panic!("[T] is not sized") }
    unsafe fn offset(_ptr: Self::Ptr, _offset: isize) -> Self::Ptr where [T]: Sized { panic!("[T] is not sized") }
    unsafe fn read(_ptr: Self::Ptr) -> [T] where [T]: Sized { panic!("[T] is not sized") }
    unsafe fn write(_ptr: Self::Ptr, _value: [T]) where [T]: Sized { panic!("[T] is not sized") }
}

pub struct SliceIter<'a, T: 'a, TLayout: ArrayLayout<T>> {
    start: TLayout::Ptr,
    end: TLayout::Ptr,
    _marker: PhantomData<&'a [T]>
}

pub struct SliceIterMut<'a, T: 'a, TLayout: ArrayLayout<T>> {
    start: TLayout::Ptr,
    end: TLayout::Ptr,
    _marker: PhantomData<&'a mut [T]>
}

impl<'a, T: 'a, TLayout: ArrayLayout<T>> Iterator for SliceIter<'a, T, TLayout> {
    type Item = Ref<'a, T, TLayout>;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { TLayout::same_ptr(self.start, self.end) } {
            None
        } else {
            unsafe {
                let value = Ref::from_raw(self.start);
                self.start = TLayout::offset(self.start, 1);
                Some(value)
            }
        }
    }
}

impl<'a, T: 'a, TLayout: ArrayLayout<T>> Iterator for SliceIterMut<'a, T, TLayout> {
    type Item = RefMut<'a, T, TLayout>;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { TLayout::same_ptr(self.start, self.end) } {
            None
        } else {
            unsafe {
                let value = RefMut::from_raw(self.start);
                self.start = TLayout::offset(self.start, 1);
                Some(value)
            }
        }
    }
}

impl<'a, T: 'a, TLayout: ArrayLayout<T>> IntoIterator for Ref<'a, [T], Slice<TLayout>> {
    type Item = Ref<'a, T, TLayout>;
    type IntoIter = SliceIter<'a, T, TLayout>;
    fn into_iter(self) -> Self::IntoIter {
        SliceIter {
            start: self.as_raw().base,
            end: unsafe { TLayout::offset(self.as_raw().base, self.as_raw().count as isize) },
            _marker: PhantomData
        }
    }
}

impl<'a, T: 'a, TLayout: ArrayLayout<T>> IntoIterator for RefMut<'a, [T], Slice<TLayout>> {
    type Item = RefMut<'a, T, TLayout>;
    type IntoIter = SliceIterMut<'a, T, TLayout>;
    fn into_iter(self) -> Self::IntoIter {
        SliceIterMut {
            start: self.as_raw().base,
            end: unsafe { TLayout::offset(self.as_raw().base, self.as_raw().count as isize) },
            _marker: PhantomData
        }
    }
}
