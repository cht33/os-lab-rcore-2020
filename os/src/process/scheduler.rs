use super::Tid;
use alloc::vec::Vec;

pub trait Scheduler {
    fn push(&mut self, tid: Tid);
    fn pop(&mut self) -> Option<Tid>;
    fn tick(&mut self) -> bool;
    fn exit(&mut self, tid: Tid);
    fn set_priority(&mut self, priority: usize, tid: Tid);
}

#[derive(Default)]
struct RRInfo {
    time: usize,
    prev: usize,
    next: usize,
}

pub struct RRScheduler {
    threads: Vec<RRInfo>,
    max_time: usize,
    current: usize,
}

impl RRScheduler {
    pub fn new(max_time_slice: usize) -> Self {
        let mut rr = RRScheduler {
            threads: Vec::default(),
            max_time: max_time_slice,
            current: 0,
        };
        rr.threads.push(RRInfo {
            time: 0,
            prev: 0,
            next: 0,
        });
        rr
    }
}
impl Scheduler for RRScheduler {
    fn push(&mut self, tid: Tid) {
        let tid = tid + 1;
        if self.threads.len() <= tid {
            self.threads.resize_with(tid + 1, Default::default);
        }

        if self.threads[tid].time == 0 {
            self.threads[tid].time = self.max_time;
        }

        let prev = self.threads[0].prev;
        self.threads[prev].next = tid;
        self.threads[tid].prev = prev;
        self.threads[0].prev = tid;
        self.threads[tid].next = 0;
    }

    fn pop(&mut self) -> Option<Tid> {
        let ret = self.threads[0].next;
        if ret != 0 {
            let next = self.threads[ret].next;
            let prev = self.threads[ret].prev;
            self.threads[next].prev = prev;
            self.threads[prev].next = next;
            self.threads[ret].prev = 0;
            self.threads[ret].next = 0;
            self.current = ret;
            Some(ret - 1)
        } else {
            None
        }
    }

    // 当前线程的可用时间片 -= 1
    fn tick(&mut self) -> bool {
        let tid = self.current;
        if tid != 0 {
            self.threads[tid].time -= 1;
            self.threads[tid].time == 0
        } else {
            true
        }
    }

    fn exit(&mut self, tid: Tid) {
        if self.current == tid + 1 {
            self.current = 0;
        }
    }

    fn set_priority(&mut self, _priority: usize, _tid: Tid) {}
}

const BIG_STRIDE: usize = (1 << 31) - 1;

#[derive(Debug)]
struct SInfo {
    stride: usize,
    pass: usize,
}

impl SInfo {
    fn update(&mut self) {
        self.stride += self.pass;
    }
}

impl Default for SInfo {
    fn default() -> SInfo {
        SInfo {stride: 0, pass: BIG_STRIDE}
    }
}

#[derive(Default)]
pub struct StrideScheduler {
    threads: Vec<SInfo>, // 每个thread的信息
    pq: Vec<Tid>,        // tid的优先队列(序号从1开始)
    len: usize,          // 优先队列的长度
    current: usize,      // 正在执行的线程
}

impl StrideScheduler {
    pub fn new() -> Self {
        let mut scheduler = StrideScheduler::default();
        scheduler.threads.push(SInfo::default());
        scheduler.pq.push(0);
        scheduler.len = 1;
        scheduler
    }
}

// 优先队列的几个辅助函数
impl StrideScheduler {
    // 向下更新
    fn go_down(&mut self, mut k: usize) {
        while k*2 < self.len {
            let mut j = 2*k;
            if j+1 < self.len && self.less(j + 1, j) { j += 1; };
            if self.less(k, j) { break; }
            self.pq.swap(k, j);
            k = j;
        }
    }

    // 向上更新
    fn go_up(&mut self, mut k: usize) {
        while k > 1 && self.less(k, k / 2) {
            self.pq.swap(k, k / 2);
            k /= 2;
        }
    }

    // 比较两个节点的stride大小
    fn less(&self, i: usize, j: usize) -> bool {
        self.threads[self.pq[i]].stride < self.threads[self.pq[j]].stride
    }

    // 插入新元素
    fn insert(&mut self, tid: Tid) {
        if self.len == self.pq.len() {
            self.pq.push(tid);
        } else {
            self.pq[self.len] = tid;
        }
        self.go_up(self.len);
        self.len += 1;
    }

    // 删除最小元素(堆顶)
    fn del_min(&mut self) -> Tid {
        let tid = self.pq[1];
        self.len -= 1;
        self.pq[1] = self.pq[self.len];
        self.go_down(1);
        tid
    }

    fn empty(&self) -> bool { self.len == 1 }

    // 按值删除元素(即删除当前tid)
    fn del_by_val(&mut self, tid: Tid) {
        let mut idx = 0;
        for i in 1..self.len {
            if self.pq[i] == tid {
                idx = i;
                break;
            }
        }
        if idx == 0 { return; }
        self.len -= 1;
        self.pq.swap(idx, self.len);
        self.go_up(idx);
        self.go_down(idx);
    }
}

impl Scheduler  for StrideScheduler {
    fn push(&mut self, tid: Tid) {
        let tid = tid + 1;
        if self.threads.len() <= tid {
            self.threads.resize_with(tid + 1, SInfo::default);
        }
        self.insert(tid);
    }

    fn pop(&mut self) -> Option<Tid> {
        if self.empty() { None }
        else {
            let tid = self.del_min();
            self.current = tid;
            Some(tid - 1)
        }
    }

    fn tick(&mut self) -> bool {
        let tid = self.current;
        if tid == 0 { true }
        else {
            self.threads[tid].update();
            let top = self.pq[1];
            self.threads[tid].stride > self.threads[top].stride
        }
    }

    fn exit(&mut self, tid: Tid) {
        let tid = tid + 1;
        if self.current == tid {
            self.current = 0;
        } else {
            self.del_by_val(tid);
        }
    }

    fn set_priority(&mut self, priority: usize, tid: Tid) {
        self.threads[tid + 1].pass = BIG_STRIDE / priority;
    }
}
