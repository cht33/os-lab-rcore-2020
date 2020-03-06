### Lab1实验报告
> 计73 陈海天 2016010106

#### Q1. 详细描述 rcore 中处理中断异常的流程（从异常的产生开始）。
答：
1. 硬件响应中断异常，自动设置中断异常相关的寄存器。
2. 因为处理模式设置为Direct，会直接跳转的中断处理程序的地址，就是__alltraps的地址。
3. 在__alltraps函数中，先保存程序运行的上下文环境。然后跳转到os实现的rust_trap函数进一步处理。
4. 在rust_trap中函数对不同的中断异常进行不同的处理。
5. 回到__alltraps函数中，恢复程序运行的上下文环境。

#### Q2. 对于任何中断，__alltraps 中都需要保存所有寄存器吗？请说明理由。
答： 不需要。应该保存处理函数中用到的寄存器即可。比如时钟中断只用到了`a1 a2 a3 a4`这四个寄存器，那么只保存这四个寄存器即可。
现在的实现是因为用的direct模式，为了方便和易于扩展所以保存了所有的寄存器。如果分拆开成向量中断模式，每个中断直接跳转到自己处理程序，那么可以简化寄存器的保存。

#### Q3. 编程：在任意位置触发一条非法指令异常（如：mret），在 rust_trap 中捕获并对其进行处理（简单 print & panic 即可）。

在rust_main中加入一条mret语句：
```rust
pub extern "C" fn rust_main() -> ! {
    ...
    crate::interrupt::init();
    unsafe { // 手动添加mret语句
        asm!("mret"::::"volatile");
    }
    ...
}
```
在rust_trap函数中增加相应的处理分支：
```rust
pub fn rust_trap(tf: &mut TrapFrame) {
    match tf.scause.cause() {
        // 简单的panic
        Trap::Exception(Exception::IllegalInstruction) => panic!("illegal instruction!"),
        ...
    }
}
```
运行的输出结果：
```
PMP0: 0x0000000080000000-0x000000008001ffff (A)
PMP1: 0x0000000000000000-0xffffffffffffffff (A,R,W,X)
switch satp from 0x8000000000080255 to 0x800000000008100c
++++ setup memory!    ++++
++++ setup interrupt! ++++
sbi_emulate_csr_read: hartid0: invalid csr_num=0x302
panicked at 'illegal instruction!', src/interrupt.rs:49:59
```
可见成功捕获并处理。
