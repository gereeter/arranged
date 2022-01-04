use core::alloc::Layout;
use core::ptr::NonNull;

use core::marker::PhantomData;

use reference::{Ref, RefMut};
use layouts::slice::{Slice, SlicePtr};
use layouts::ArrayLayout;

pub struct Parallel<Innards> {
    _marker: PhantomData<Innards>
}

unsafe impl<L, R, LLayout, RLayout> ArrayLayout<(L, R)> for Parallel<(LLayout, RLayout)> where LLayout: ArrayLayout<L>, RLayout: ArrayLayout<R> {
    type Ptr = (LLayout::Ptr, RLayout::Ptr);
    type ArrayInfo = (LLayout::ArrayInfo, RLayout::ArrayInfo, usize);

    fn layout_array(count: usize) -> (Layout, Self::ArrayInfo) {
        let (l_layout, l_info) = LLayout::layout_array(count);
        let (r_layout, r_info) = RLayout::layout_array(count);
        let (combined_layout, offset) = l_layout.extend(r_layout).expect("Overflow in combining array layouts");
        (combined_layout, (l_info, r_info, offset))
    }

    unsafe fn from_flat_ptr(ptr: NonNull<u8>, (l_info, r_info, offset): Self::ArrayInfo) -> Self::Ptr {
        (LLayout::from_flat_ptr(ptr, l_info), RLayout::from_flat_ptr(NonNull::new_unchecked(ptr.as_ptr().offset(offset as isize)), r_info))
    }

    unsafe fn initialize((l_ptr, r_ptr): Self::Ptr, count: usize) {
        LLayout::initialize(l_ptr, count);
        RLayout::initialize(r_ptr, count);
    }

    unsafe fn base_ptr((l_ptr, _): Self::Ptr, (l_info, _, _): Self::ArrayInfo) -> NonNull<u8> {
        LLayout::base_ptr(l_ptr, l_info)
    }

    fn dangling() -> Self::Ptr {
        (LLayout::dangling(), RLayout::dangling())
    }

    unsafe fn offset((l_ptr, r_ptr): Self::Ptr, offset: isize) -> Self::Ptr {
        (LLayout::offset(l_ptr, offset), RLayout::offset(r_ptr, offset))
    }

    unsafe fn same_ptr((l_ptr1, r_ptr1): Self::Ptr, (l_ptr2, r_ptr2): Self::Ptr) -> bool {
        if core::mem::size_of::<L>() > 0 {
            LLayout::same_ptr(l_ptr1, l_ptr2)
        } else {
            RLayout::same_ptr(r_ptr1, r_ptr2)
        }
    }

    unsafe fn read((l_ptr, r_ptr): Self::Ptr) -> (L, R) {
        (LLayout::read(l_ptr), RLayout::read(r_ptr))
    }

    unsafe fn write((l_ptr, r_ptr): Self::Ptr, (l_value, r_value): (L, R)) {
        LLayout::write(l_ptr, l_value);
        RLayout::write(r_ptr, r_value);
    }

    unsafe fn drop_in_place((l_ptr, r_ptr): Self::Ptr) {
        LLayout::drop_in_place(l_ptr);
        RLayout::drop_in_place(r_ptr);
    }

    unsafe fn copy_one_nonoverlapping((l_src, r_src): Self::Ptr, (l_dest, r_dest): Self::Ptr) {
        LLayout::copy_one_nonoverlapping(l_src, l_dest);
        RLayout::copy_one_nonoverlapping(r_src, r_dest);
    }

    unsafe fn swap_one_nonoverlapping((l_ptr1, r_ptr1): Self::Ptr, (l_ptr2, r_ptr2): Self::Ptr) {
        LLayout::swap_one_nonoverlapping(l_ptr1, l_ptr2);
        RLayout::swap_one_nonoverlapping(r_ptr1, r_ptr2);
    }

    unsafe fn copy_leftwards((l_src, r_src): Self::Ptr, (l_dest, r_dest): Self::Ptr, count: usize) {
        LLayout::copy_leftwards(l_src, l_dest, count);
        RLayout::copy_leftwards(r_src, r_dest, count);
    }

    unsafe fn copy_rightwards((l_src, r_src): Self::Ptr, (l_dest, r_dest): Self::Ptr, count: usize) {
        LLayout::copy_rightwards(l_src, l_dest, count);
        RLayout::copy_rightwards(r_src, r_dest, count);
    }

    unsafe fn copy_nonoverlapping((l_src, r_src): Self::Ptr, (l_dest, r_dest): Self::Ptr, count: usize) {
        LLayout::copy_nonoverlapping(l_src, l_dest, count);
        RLayout::copy_nonoverlapping(r_src, r_dest, count);
    }

    unsafe fn swap_nonoverlapping((l_ptr1, r_ptr1): Self::Ptr, (l_ptr2, r_ptr2): Self::Ptr, count: usize) {
        LLayout::swap_nonoverlapping(l_ptr1, l_ptr2, count);
        RLayout::swap_nonoverlapping(r_ptr1, r_ptr2, count);
    }
}

impl<'a, L, R, LLayout, RLayout> Ref<'a, [(L, R)], Slice<Parallel<(LLayout, RLayout)>>> where LLayout: ArrayLayout<L>, RLayout: ArrayLayout<R> {
    pub fn unzip(self) -> (Ref<'a, [L], Slice<LLayout>>, Ref<'a, [R], Slice<RLayout>>) {
        unsafe { (
            Ref::from_raw(SlicePtr::from_raw_parts(
                self.as_ptr().0,
                self.len()
            )),
            Ref::from_raw(SlicePtr::from_raw_parts(
                self.as_ptr().1,
                self.len()
            ))
        ) }
    }
}

impl<'a, L, R, LLayout, RLayout> RefMut<'a, [(L, R)], Slice<Parallel<(LLayout, RLayout)>>> where LLayout: ArrayLayout<L>, RLayout: ArrayLayout<R> {
    pub fn unzip(self) -> (RefMut<'a, [L], Slice<LLayout>>, RefMut<'a, [R], Slice<RLayout>>) {
        unsafe { (
            RefMut::from_raw(SlicePtr::from_raw_parts(
                self.as_ptr().0,
                self.len()
            )),
            RefMut::from_raw(SlicePtr::from_raw_parts(
                self.as_ptr().1,
                self.len()
            ))
        ) }
    }
}
