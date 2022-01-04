use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use layouts::{ArrayLayout, Flat, Extra};

pub struct Ref<'a, T: ?Sized + 'a, TLayout = Flat> where TLayout: ArrayLayout<T> {
    ptr: TLayout::Ptr,
    _marker: PhantomData<&'a T>
}

pub struct RefMut<'a, T: ?Sized + 'a, TLayout = Flat> where TLayout: ArrayLayout<T> {
    ptr: TLayout::Ptr,
    _marker: PhantomData<&'a mut T>
}

impl<'a, T: ?Sized, TLayout: ArrayLayout<T>> Copy for Ref<'a, T, TLayout> { }
impl<'a, T: ?Sized, TLayout: ArrayLayout<T>> Clone for Ref<'a, T, TLayout> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T> Deref for Ref<'a, T, Flat> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<'a, T> Deref for RefMut<'a, T, Flat> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<'a, T> DerefMut for RefMut<'a, T, Flat> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.as_mut()
        }
    }
}

impl<'a, T> Deref for Ref<'a, (), Extra<T>> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<'a, T> Deref for RefMut<'a, (), Extra<T>> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<'a, T> DerefMut for RefMut<'a, (), Extra<T>> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.as_mut()
        }
    }
}

impl<'a, T: ?Sized, TLayout: ArrayLayout<T>> Ref<'a, T, TLayout> {
    pub unsafe fn from_raw(ptr: TLayout::Ptr) -> Self {
        Ref {
            ptr: ptr,
            _marker: PhantomData
        }
    }

    pub fn as_raw(&self) -> TLayout::Ptr {
        self.ptr
    }

    pub fn reborrow<'b>(&'b self) -> Ref<'b, T, TLayout> {
        unsafe { Ref::from_raw(self.as_raw()) }
    }
}

impl<'a, T: ?Sized, TLayout: ArrayLayout<T>> RefMut<'a, T, TLayout> {
    pub unsafe fn from_raw(ptr: TLayout::Ptr) -> Self {
        RefMut {
            ptr: ptr,
            _marker: PhantomData
        }
    }

    pub fn as_raw(&self) -> TLayout::Ptr {
        self.ptr
    }

    pub fn reborrow<'b>(&'b self) -> Ref<'b, T, TLayout> {
        unsafe { Ref::from_raw(self.as_raw()) }
    }

    pub fn reborrow_mut<'b>(&'b mut self) -> RefMut<'b, T, TLayout> {
        unsafe { RefMut::from_raw(self.as_raw()) }
    }
}

impl<'a, T> Ref<'a, T, Flat> {
    pub fn from_flat(reference: &'a T) -> Self {
        unsafe {
            Ref::from_raw(NonNull::new_unchecked(reference as *const _ as *mut _))
        }
    }
}

impl<'a, T> RefMut<'a, T, Flat> {
    pub fn from_flat(reference: &'a mut T) -> Self {
        unsafe {
            RefMut::from_raw(NonNull::new_unchecked(reference as *mut _))
        }
    }
}

impl<'a, T, TLayout: ArrayLayout<T>> RefMut<'a, T, TLayout> {
    pub fn replace(reference: Self, value: T) -> T {
        unsafe {
            let old = TLayout::read(reference.ptr);
            TLayout::write(reference.ptr, value);
            old
        }
    }
}
