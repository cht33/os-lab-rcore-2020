### Lab3实验报告
> 计73 陈海天 2016010106

#### Q1. 现有页面替换算法框架的实现存在问题，请解释为什么，并提供你的解决方案。
答：
有两个问题：
1. 异处理过于粗糙，现在只要是page fault就会触发swap page，但实际触发page fault的原因有很多，
   ```rust
   // 比如没有访问非法地址的情况下，这一句会导致内核直接崩溃
    let entry = pg_table.ref_entry(page.clone()).unwrap();
   ```
   解决方案是在interrupt中更精细地处理page_fault。
2. PageReplace中的do_fault函数没有调用push_frame函数加入新swap in的page。这样会导致在swap一定数量的page之后将没有page可换。事实上现在的lab3的test跑完之后刚好就处于没有page可以换出的状况，这个时候再调用do_pgfault就会出错。解决方案是重新实现do_pgfault函数，根据不同的情况来判断是否要调用push_frame。

#### Q2. 编程解决：实现时钟页面替换算法。

主要就是修改了choose_victim函数，按部就班实现时钟替换算法。
```rust
    println!("push vaddr: {:#x?}", vaddr);
    self.frames.insert(self.p, (vaddr, pt));
    self.p += 1;
}
fn choose_victim(&mut self) -> Option<(usize, Arc<Mutex<PageTableImpl>>)> {
    // self.p是时钟替换算法中的指针位置
    let len = self.frames.len();
    loop {
        self.p = self.p % len;
        let (vaddr, pt) = self.frames.get(self.p).unwrap();
        if let Some(entry) = pt.lock().get_entry(*vaddr) {
            if entry.accessed() {
                entry.clear_accessed();
                self.p += 1;
            } else { break; }
        } else { panic!("invalid page when choosing victim") }
    }
    self.frames.remove(self.p)
}
```
存在的一个问题是时钟替换算法会修改access位，但是这样在swap_out_one函数中对于写回到磁盘的判断就会出现错误，所以我在fifo.rs中重写了这个函数，将判断条件改成了在dirty时才会写回到磁盘。
```rust
fn swap_out_one(&mut self) -> Option<Frame> {
            ...
            // 从if entry.accessed()改成了if entry.dirty()
            if entry.dirty() {
                // 写回到磁盘
            }
            ...
}
```