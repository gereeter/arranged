#![feature(allocator_api, dropck_eyepatch)]
#![no_std]
extern crate alloc;
extern crate arranged;

use alloc::alloc::Global;
use core::alloc::Alloc;
use core::cmp;
use core::marker::PhantomData;
use core::ops::{RangeBounds, Bound};

use arranged::{Ref, RefMut};
use arranged::layouts::{Flat, Slice};
use arranged::layouts::ArrayLayout;
use arranged::layouts::slice::{SlicePtr, SliceIter, SliceIterMut};

pub struct AVec<T, TLayout = Flat, A = Global> where TLayout: ArrayLayout<T>, A: Alloc {
    ptr: TLayout::Ptr,
    count: usize,
    capacity: usize,
    allocator: A,
    _marker: PhantomData<T>
}

unsafe impl<#[may_dangle] T, TLayout: ArrayLayout<T>, A: Alloc> Drop for AVec<T, TLayout, A> {
    fn drop(&mut self) {
        self.clear();
        let (layout, array_info) = TLayout::layout_array(self.capacity);
        unsafe {
            self.allocator.dealloc(TLayout::base_ptr(self.ptr, array_info), layout);
        }
    }
}

pub struct Drain<'a, T: 'a, TLayout> where TLayout: ArrayLayout<T> {
    vec_count: &'a mut usize,
    shift_count: usize,
    full_range_start: TLayout::Ptr,
    full_range_end: TLayout::Ptr,
    range_start: TLayout::Ptr,
    range_end: TLayout::Ptr,
    _marker: PhantomData<&'a mut [T]>
}

impl<'a, T, TLayout: ArrayLayout<T>> Drop for Drain<'a, T, TLayout> {
    fn drop(&mut self) {
        unsafe {
            // Drop the remaining elements
            while !TLayout::same_ptr(self.range_start, self.range_end) {
                TLayout::drop_in_place(self.range_start);
                self.range_start = TLayout::offset(self.range_start, 1);
            }

            // Shift the remaining elements into place
            TLayout::copy_leftwards(self.full_range_end, self.full_range_start, self.shift_count);

            // And finally let the AVec know that the remaining elements are available
            *self.vec_count += self.shift_count;
        }
    }
}

impl<'a, T, TLayout: ArrayLayout<T>> Iterator for Drain<'a, T, TLayout> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if unsafe { TLayout::same_ptr(self.range_start, self.range_end) } {
            None
        } else {
            unsafe {
                let ret = TLayout::read(self.range_start);
                self.range_start = TLayout::offset(self.range_start, 1);
                Some(ret)
            }
        }
    }
}

impl<'a, T, TLayout: ArrayLayout<T>> DoubleEndedIterator for Drain<'a, T, TLayout> {
    fn next_back(&mut self) -> Option<T> {
        if unsafe { TLayout::same_ptr(self.range_start, self.range_end) } {
            None
        } else {
            unsafe {
                self.range_end = TLayout::offset(self.range_end, -1);
                let ret = TLayout::read(self.range_end);
                Some(ret)
            }
        }
    }
}

pub struct DrainFilter<'a, F, T: 'a, TLayout> where TLayout: ArrayLayout<T>, F: FnMut(RefMut<T, TLayout>) -> bool {
    vec_count: &'a mut usize,
    base_ptr: TLayout::Ptr,
    read_count: usize,
    write_count: usize,
    total_count: usize,
    predicate: F,
    _marker: PhantomData<&'a mut [T]>
}

impl<'a, F, T, TLayout: ArrayLayout<T>> Iterator for DrainFilter<'a, F, T, TLayout> where F: FnMut(RefMut<T, TLayout>) -> bool {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        while self.read_count < self.total_count {
            let mut elem_ref: RefMut<'_, T, TLayout> = unsafe {
                RefMut::from_raw(TLayout::offset(self.base_ptr, self.read_count as isize))
            };
            self.read_count += 1;
            if (self.predicate)(elem_ref.reborrow_mut()) {
                return Some(unsafe {
                    TLayout::read(elem_ref.as_raw())
                });
            } else {
                let target_idx = self.write_count;
                self.write_count += 1;
                if self.write_count != self.read_count {
                    unsafe {
                        TLayout::copy_one_nonoverlapping(elem_ref.as_raw(), TLayout::offset(self.base_ptr, target_idx as isize));
                    }
                }
            }
        }
        None
    }
}

impl<'a, F: FnMut(RefMut<T, TLayout>) -> bool, T, TLayout: ArrayLayout<T>> Drop for DrainFilter<'a, F, T, TLayout> {
    fn drop(&mut self) {
        // FIXME: use drop_in_place
        for _ in &mut *self { }
        *self.vec_count = self.write_count;
    }
}

impl<T, TLayout: ArrayLayout<T>> AVec<T, TLayout, Global> {
    pub fn new() -> Self {
        AVec {
            ptr: TLayout::dangling(),
            count: 0,
            capacity: 0,
            allocator: Global,
            _marker: PhantomData
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (layout, info) = TLayout::layout_array(capacity);
        if layout.size() == 0 {
            return Self::new();
        }

        let allocation = unsafe { Global.alloc(layout).unwrap() };
        let ptr = unsafe { TLayout::from_flat_ptr(allocation, info) };
        unsafe { TLayout::initialize(ptr, capacity); }
        AVec {
            ptr: ptr,
            count: 0,
            capacity: capacity,
            allocator: Global,
            _marker: PhantomData
        }
    }
}

impl<T, TLayout: ArrayLayout<T>, A: Alloc> AVec<T, TLayout, A> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub unsafe fn set_len(&mut self, length: usize) {
        self.count = length;
    }

    pub fn as_slice<'a>(&'a self) -> Ref<'a, [T], Slice<TLayout>> {
        unsafe {
            Ref::from_raw(SlicePtr::from_raw_parts(
                self.ptr,
                self.count
            ))
        }
    }

    pub fn as_mut_slice<'a>(&'a mut self) -> RefMut<'a, [T], Slice<TLayout>> {
        unsafe {
            RefMut::from_raw(SlicePtr::from_raw_parts(
                self.ptr,
                self.count
            ))
        }
    }

    pub fn iter<'a>(&'a self) -> SliceIter<'a, T, TLayout> {
        self.as_slice().into_iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> SliceIterMut<'a, T, TLayout> {
        self.as_mut_slice().into_iter()
    }

    pub fn drain<'a, R>(&'a mut self, range: R) -> Drain<'a, T, TLayout> where R: RangeBounds<usize> {
        // For the half-open interval of the range
        let start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0
        };
        let end = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.len()
        };

        assert!(start <= end);
        assert!(end <= self.len());

        unsafe {
            // PPYP
            let total_count = self.len();
            self.set_len(start);
            let start_ptr = TLayout::offset(self.ptr, start as isize);
            let end_ptr = TLayout::offset(self.ptr, end as isize);
            Drain {
                // For shifting on drop
                vec_count: &mut self.count,
                shift_count: total_count - end,
                full_range_start: start_ptr,
                full_range_end: end_ptr,
                range_start: start_ptr,
                range_end: end_ptr,
                _marker: PhantomData
            }
        }
    }

    pub fn drain_filter<'a, F>(&'a mut self, predicate: F) -> DrainFilter<'a, F, T, TLayout> where F: for<'b> FnMut(RefMut<'b, T, TLayout>) -> bool {
        let total_count = self.len();
        unsafe {
            // PPYP
            self.set_len(0);
        }

        DrainFilter {
            vec_count: &mut self.count,
            base_ptr: self.ptr,
            read_count: 0,
            write_count: 0,
            total_count: total_count,
            predicate: predicate,
            _marker: PhantomData
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            unsafe {
                self.count -= 1;
                Some(TLayout::read(TLayout::offset(self.ptr, self.count as isize)))
            }
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        unsafe {
            self.count -= 1;
            let ret = TLayout::read(TLayout::offset(self.ptr, index as isize));
            if index != self.count {
                TLayout::copy_one_nonoverlapping(TLayout::offset(self.ptr, self.count as isize), TLayout::offset(self.ptr, index as isize));
            }
            ret
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        unsafe {
            self.count -= 1;
            let ret = TLayout::read(TLayout::offset(self.ptr, index as isize));
            TLayout::copy_leftwards(TLayout::offset(self.ptr, (index + 1) as isize), TLayout::offset(self.ptr, index as isize), self.count - index);
            ret
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let count = self.len();
            self.set_len(0);
            Slice::<TLayout>::drop_in_place(SlicePtr::from_raw_parts(self.ptr, count));
        }
    }

    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            unsafe {
                let count = self.len();
                self.set_len(len);
                Slice::<TLayout>::drop_in_place(SlicePtr::from_raw_parts(self.ptr, count - len));
            }
        }
    }

    pub fn push(&mut self, value: T) {
        self.reserve_one();

        unsafe {
            TLayout::write(TLayout::offset(self.ptr, self.count as isize), value);
            self.count += 1;
        }
    }

    fn reserve_one(&mut self) {
        if self.count >= self.capacity {
            let new_capacity = if self.capacity == 0 {
                2
            } else {
                self.capacity * 2
            };
            let (new_layout, new_info) = TLayout::layout_array(new_capacity);
            let new_allocation = unsafe { self.allocator.alloc(new_layout).unwrap() };
            let new_ptr = unsafe { TLayout::from_flat_ptr(new_allocation, new_info) };
            unsafe { TLayout::initialize(new_ptr, new_capacity); }

            if self.count != 0 {
                unsafe {
                    TLayout::copy_nonoverlapping(self.ptr, new_ptr, self.count);

                    let (old_layout, old_info) = TLayout::layout_array(self.capacity);
                    self.allocator.dealloc(TLayout::base_ptr(self.ptr, old_info), old_layout);
                }
            }

            self.ptr = new_ptr;
            self.capacity = new_capacity;
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.count + additional > self.capacity {
            let new_capacity = cmp::max(self.count + additional, self.capacity * 2);
            let (new_layout, new_info) = TLayout::layout_array(new_capacity);
            let new_allocation = unsafe { self.allocator.alloc(new_layout).unwrap() };
            let new_ptr = unsafe { TLayout::from_flat_ptr(new_allocation, new_info) };
            unsafe { TLayout::initialize(new_ptr, new_capacity); }

            if self.count != 0 {
                unsafe {
                    TLayout::copy_nonoverlapping(self.ptr, new_ptr, self.count);

                    let (old_layout, old_info) = TLayout::layout_array(self.capacity);
                    self.allocator.dealloc(TLayout::base_ptr(self.ptr, old_info), old_layout);
                }
            }

            self.ptr = new_ptr;
            self.capacity = new_capacity;
        }
    }

    pub fn append<OtherA: Alloc>(&mut self, other: &mut AVec<T, TLayout, OtherA>) {
        self.reserve(other.count);
        unsafe {
            TLayout::copy_nonoverlapping(other.ptr, TLayout::offset(self.ptr, self.count as isize), other.count);
            other.set_len(0);
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());
        self.reserve_one();
        unsafe {
            TLayout::copy_rightwards(TLayout::offset(self.ptr, index as isize), TLayout::offset(self.ptr, (index + 1) as isize), self.len() - index);
            TLayout::write(TLayout::offset(self.ptr, index as isize), value);
            self.count += 1;
        }
    }

    pub fn retain<F: for<'a> FnMut(Ref<'a, T, TLayout>) -> bool>(&mut self, mut func: F) {
        self.drain_filter(move |x| !func(x.reborrow()));
    }
}

#[cfg(test)]
mod tests {
    use super::AVec;
    use arranged::layouts::{Flat, Parallel, PackedBits};

    #[test]
    fn push_pop() {
        let mut vec: AVec<(u64, u64), Parallel<(Flat, Flat)>> = AVec::new();
        vec.push((1, 2));
        vec.push((3, 4));
        vec.push((5, 6));
        vec.push((7, 8));
        vec.push((9, 10));

        let mut left_match = 1;
        for left_val in vec.as_slice().unzip().0 {
            assert_eq!(left_match, *left_val);
            left_match += 2;
        }
        let mut right_match = 2;
        for right_val in vec.as_slice().unzip().1 {
            assert_eq!(right_match, *right_val);
            right_match += 2;
        }

        assert_eq!(vec.pop(), Some((9, 10)));
        assert_eq!(vec.pop(), Some((7, 8)));
        assert_eq!(vec.pop(), Some((5, 6)));
        assert_eq!(vec.pop(), Some((3, 4)));
        assert_eq!(vec.pop(), Some((1, 2)));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn bitvec() {
        let mut vec: AVec<bool, PackedBits<Flat>> = AVec::new();
        for _ in 0..1000 {
            vec.push(false);
            vec.push(true);
            vec.push(true);
            vec.push(false);
            vec.push(false);
            vec.push(true);
            vec.push(false);
        }

        for _ in 0..1000 {
            assert_eq!(vec.pop(), Some(false));
            assert_eq!(vec.pop(), Some(true));
            assert_eq!(vec.pop(), Some(false));
            assert_eq!(vec.pop(), Some(false));
            assert_eq!(vec.pop(), Some(true));
            assert_eq!(vec.pop(), Some(true));
            assert_eq!(vec.pop(), Some(false));
        }
        assert_eq!(vec.pop(), None);
    }
}
