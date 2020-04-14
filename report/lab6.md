### Lab6实验报告
> 计73 陈海天 2016010106

#### Q1.编程：将 Round Robin 调度算法 替换为 Stride 调度算法 。
###### 实现简述
算法原理即ucore文档中的描述，不再赘述。对于其中的思考题：BIG_STRIDE的取值范围，简单计算可知取值不能超过当前的数据类型的INT_MAX，比如用32整数表示，那么不能超过$2^31-1$，这是由无符号数和有符号数转换的规则决定的。另一方面BIG_STRIDE应该尽可能大，这样能减少整数除法带来的误差，也能容纳更多优先级。
具体实现上使用了一个简单的优先队列，这样时间复杂度和空间复杂度都比较好。其中`push, pop`就是优先队列插入删除，时间$O(logn)$；`tick, set_priority`时间$O(1)$；`exit`需删除指定元素，时间$O(n)$。空间复杂度是$3n*sizeof(usize)$
```rust
struct SInfo {
    stride: usize,
    pass: usize,
}
pub struct StrideScheduler {
    threads: Vec<SInfo>, // 每个thread的信息
    pq: Vec<Tid>,        // tid的优先队列(序号从1开始)
    len: usize,          // 优先队列的长度
    current: usize,      // 正在执行的线程
}
// 优先队列的几个辅助函数
impl StrideScheduler {
    fn go_down(&mut self, mut k: usize) { ... }  // 向下更新
    fn go_up(&mut self, mut k: usize) { ... }  // 向上更新
    fn less(&self, i: usize, j: usize) -> bool { ... } // 比较两个节点的stride大小
    fn insert(&mut self, tid: Tid) { ... } // 插入新元素
    fn del_min(&mut self) -> Tid { ... } // 删除最小元素(堆顶)
    fn empty(&self) -> bool { self.len == 1 } // 优先队列是否为空
    fn del_by_val(&mut self, tid: Tid) { ... } // 按值删除元素(即删除当前tid)
}
// 接口实现
impl Scheduler  for StrideScheduler { ... }
}
```
其余修改即增加了相关的系统调用接口。

##### 输出结果分析
一个典型的输出如下：
```
++++ setup process!   ++++
++++ setup timer!     ++++
main: fork ok.
thread 0 exited, exit code = 0
thread 4 exited, exit code = 331600
thread 3 exited, exit code = 248000
thread 2 exited, exit code = 165200
thread 5 exited, exit code = 410000
thread 1 exited, exit code = 82800
```
关于`thread 1`用时进行归一化，得：
```
time cost:
thread 1: 1.0
thread 2: 1.995
thread 3: 2.995
thread 4: 4.005
thread 5: 4.952
```
可见所用时间与线程优先级呈比较好的正比关系。