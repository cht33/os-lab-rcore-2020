use {
    super::*,
    alloc::{collections::VecDeque, sync::Arc},
    spin::Mutex,
};

#[derive(Default)]
pub struct FifoPageReplace {
    frames: VecDeque<(usize, Arc<Mutex<PageTableImpl>>)>,
    p: usize, // 时钟替换算法的当前位置
}

impl PageReplace for FifoPageReplace {
    fn push_frame(&mut self, vaddr: usize, pt: Arc<Mutex<PageTableImpl>>) {
        println!("push vaddr: {:#x?}", vaddr);
        self.frames.push_back((vaddr, pt));
    }

    fn choose_victim(&mut self) -> Option<(usize, Arc<Mutex<PageTableImpl>>)> {
        // 选择一个已经分配的物理页帧
        let len = self.frames.len();
        loop {
            self.p = self.p % len;
            let (vaddr, pt) = self.frames.get(self.p).unwrap();
            if let Some(entry) = pt.lock().get_entry(*vaddr) {
                if entry.accessed() {
                    entry.clear_accessed();
                    self.p += 1;
                } else { break; }
            } else { panic!("invalid page when choose_victim") }
        }
        self.frames.remove(self.p)
    }

    fn tick(&self) {}
}
