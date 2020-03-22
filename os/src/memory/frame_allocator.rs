use crate::consts::MAX_PHYSICAL_PAGES;
use spin::Mutex;

pub struct SegmentTreeAllocator {
    a: [u8; MAX_PHYSICAL_PAGES << 1], // root is a[1]
    m: usize,       // m is the leftmost leaf's idx in the segment tree
    n: usize,       // n is the size of the ppn pages
    offset: usize,  // the offset of the ppn pages
}

// some help macro
macro_rules! l_child { ($node:expr) => (($node) << 1) }
macro_rules! r_child { ($node:expr) => ((($node) << 1) | 1) }
macro_rules! parent  { ($node:expr) => (($node) >> 1) }

impl SegmentTreeAllocator {
    // init with ppn range [l, r)
    pub fn init(&mut self, l: usize, r: usize) {
        self.offset = l;
        self.n = r - l;
        self.m = 1;
        // m = 2^k, where 2^(k-1) < n <= 2^k
        while self.m < self.n { self.m = self.m << 1; }
        // init all the inner nodes with 1
        for i in 1..(self.m << 1) { self.a[i] = 1; }
        // init all the leafs with 0
        for i in 0..self.n { self.a[self.m + i] = 0; }
        // update the inner nodes
        for i in (1..self.m).rev() {
            self.a[i] = self.a[l_child!(i)] & self.a[r_child!(i)];
        }
    }

    pub fn alloc(&mut self) -> usize {
        // assume that we never run out of physical memory
        if self.a[1] == 1 {
            panic!("physical memory depleted!");
        }
        let mut p = 1;
        while p < self.m { // find an empty node
            if self.a[l_child!(p)] == 0 { p = l_child!(p) }
            else { p = r_child!(p) }
        }
        self.update(p, 1);
        // p - m + offset is the idx in the range [l, r)
        p + self.offset - self.m
    }

    pub fn dealloc(&mut self, n: usize) {
        let p = n + self.m - self.offset;
        assert!(self.a[p] == 1);
        self.update(p, 0);
    }

    #[inline(always)]
    fn update(&mut self, mut p: usize, value: u8) {
        self.a[p] = value;
        while p != 1 {
            p = parent!(p);
            self.a[p] = self.a[l_child!(p)] & self.a[r_child!(p)];
        }
    }
}

pub static SEGMENT_TREE_ALLOCATOR: Mutex<SegmentTreeAllocator> = Mutex::new(SegmentTreeAllocator {
    a: [0; MAX_PHYSICAL_PAGES << 1],
    m: 0,
    n: 0,
    offset: 0,
});
