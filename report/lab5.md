### Lab5实验报告
> 计73 陈海天 2016010106

#### Q1.编程实现：为 rcore 增加 sys_fork 。

答：复制出一个新线程(进程)具体需要以下内容：
- 一个新的内核栈
- 一份线程上下文
- 一套新的页表，虚拟地址映射不变，物理页都复制一遍
- 子线程自己用于返回的TrapFrame

##### 具体实现如下：
1. 在系统调用，异常处理等部分中加入fork相关的接口。最终调用到`Thread`的`fork`函数。
2. 在`fork`函数中新申请一个内核栈；申请一个空的页表；把当前线程的`MemorySet`和页表复制一份，作为新线程的资源。具体过程如下：(仿照rcore的实现)
   - `MemorySet`添加`clone`函数。具体作用：创建新的页表，对每个`MemoryArea`调用`clone_map_all`函数来复制相应的物理页和页表内容。最后构造新的`MemorySet`返回。
   - `MemoryArea`添加`clone_map_all`函数：调用相应的`MemoryHandler`的`clone_map`函数，逐一复制src页表中的所有page到dst页表中。
   - `MemoryHandler`添加`clone_map`接口：复制page对应的映射关系及管理相应的物理页内容。
   - `Linear Handler`管理的是内核代码段，一则无需复制一份，所有线程可以共享，二则因为是线性偏移，一个va不可能对应到两个pa，也无法复制物理页。所以`clone_map`中只需给dst页表添加相应的映射关系。
   - `ByFrame Handler`的`clone_map`就是真正实现物理页复制的地方。dst页表中添加映射关系后，对应的物理页是新申请的空白页，接下来把src页表中对应的物理页的内容复制到该页即可。调用提示的`get_page_slice_mut`函数进行复制。
3. 结束后得到新线程的`MemorySet`，回到`fork`函数中。调用`Context`的`new_fork`函数，接下来构造新线程的`ContextContent`：
   - `ra`: `__trapret`，因为新线程也是异常调用完返回。
   - `satp`: 新`MemorySet`的页表。
   - `s[0:12]: 全0即可。
   - tf: 复制一份父线程的tf，再把x[10]改成0。
把得到的content压入新的内核栈中，得到相应的context。
4. 新线程所需的context，kstack和memoryset都构造完毕，最后的文件资源ofile简单地复制一份父线程的即可。

测试输出：
```
I am child
ret tid is: 0
thread 2 exited, exit code = 0
I am father
ret tid is: 2
thread 1 exited, exit code = 0
I am child
ret tid is: 0
thread 3 exited, exit code = 0
I am father
ret tid is: 3
thread 0 exited, exit code = 0
```
注意到这和提供的参考输出差别较大，这是线程调度的不同导致的。而且参考输出有三个child一个father，情况比较特殊。
简单来说，上面的输出时间片较短，每fork一次就会用完并切换。参考输出的时间片很长，每个线程都一次执行完没有中断过。
```
[a]
 |\
 | \
 | [b]
 |   \
[c]  [d]
----------------
上面输出的调度情况                     线程调度栈
1. a创建,分配到tid=0                   [a]
   a fork出b, b分配到tid=1             [a][b]
   a 结束, 切换到b                     [b][a]
2. b fork出d, d分配到tid=2             [b][a][d]
   b 结束, 切换到a                     [a][d][b]
3. a fork出c, c分配到tid=3             [a][d][b][c]
   a 结束, 切换到d                     [d][b][c][a]
4. d输出child并exit                    [b][c][a]
5. b输出father并exit                   [c][a]
6. c输出child并exit                    [a]
7. a输出father并exit                   []
----------------
参考输出的调度情况                     线程调度栈
1. a创建,分配到tid=0                   [a]
   a fork出b, b分配到tid=1             [a][b]
   a fork出c, c分配到tid=2             [a][b][c]
   a输出father,并exit                  [b][c]
   此时tid=0的位置空缺出来
2. b fork出d, d分配到tid=0             [b][c][d]
   因为d为0,b误判自己是child并exit      [c][d]
3. c输出child并exit                    [d]
4. d输出child并exit                    []
```