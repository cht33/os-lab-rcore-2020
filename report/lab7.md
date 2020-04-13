### Lab7实验报告
> 计73 陈海天 2016010106

#### Q1. 编程：将在实验指导中提供 Mutex 的实现框架、sleep 的实现、spawn 的实现和哲学家就餐问题测试，请将它们复制到你的代码中，并完成 Mutex 中的 TODO 部分。
编程任务比较简短，大部分代码框架都已提供好，两个TODO分别实现如下：
```rust
fn obtain_lock(&self) {
    loop {
        let p = self.lock.get();
        if unsafe{ ! *p } {
            unsafe { *p = true; }
            break;
        } else { yield_now(); }
    }
}
```
使用轮询的方式，如果`*p == true`说明已经上锁，就调用`yield_now`主动让出cpu；否则说明可以获取该锁，修改为`true`后直接结束轮询。
```rust
fn drop(&mut self) {
    *self.lock = false;
}
```
`drop`中直接放弃锁即可。

#### Q2. 回答：Mutex 的实现中，为什么需要引入 MutexGuard ？
答：`MutexGuard`实现了对资源和锁的封装，一方面可以通过`Deref, DerefMut`方便地访问资源，另一方面在`drop`中实现了锁的自动释放，避免手动释放可能带来的问题。

#### Q3. 回答：Mutex 的实现中，为什么需要修改 yield_now 并增加 park ，如果都仍然使用旧的 yield_now 会出现什么问题？
答：新的`park`实际上就是旧的`yield_now`。
原有的实现是把当前线程状态设置为`sleeping`并让出cpu，需要被`wake_up`才能加入调度序列。
现在的`yield_now`只保留了让出cpu的部分，当前线程随时等待被调度。
如果在`obtain_lock`中使用原来的`yield_now`，那么就缺少了设置`wake_up`的回调过程，当前线程将不会再被唤醒。

#### Q4. 回答：sleep 的实现中，为什么需要在 idle_main 中增加一瞬间的中断开启，不增加这部分会出现什么问题？
答：为了在调度的时候能响应时钟中断来唤醒线程。不加这部分会出现"死锁"。
所有的哲学家都是内核线程，因为第五题的缘故，不会响应异步中断，包括了时钟中断。如果在idle中也无法响应时钟中断的话，那么当有线程ready的时候，sleep的线程永远不会被唤醒。这就可能出现sleep的线程占有资源，ready的线程拿不到资源，将一直保持ready的状态。一个典型的例子如下：
```
1 is thinking.
2 is thinking.
3 is thinking.
4 is thinking.
5 is thinking.
1 is eating, using forks: 0, 1
3 is eating, using forks: 2, 3

```
这种情况下，1和3吃完第一轮，拿着筷子在休息，但是没人叫醒他们；245不断上桌子找筷子却找不到。形成死锁。

#### Q5. 回答：在哲学家就餐测试中，为什么需要修改 spie ，如果不进行修改可能会出现什么问题？
答：保证了哲学家不会被异步中断打断就餐过程，即保证了锁操作的原子性。否则可能会出现两个哲学家都认为自己拿到了同一根筷子，同时访问资源造成冲突。