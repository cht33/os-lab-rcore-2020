### Lab8实验报告
> 计73 陈海天 2016010106

#### Q1. 编程实现：基于第九章的内容，支持 pipe ，使得给定的用户态测试程序得到正确的结果。
主要的实现内容如下：
1. 增加相关的系统调用接口。
   `sys_pipe`的功能是：为当前进程创建一个管道，并返回两个文件描述符分别代表它的读端和写端。
2. `File`中增加与pipe相关的接口。
   ```rust
    pub struct File {
        ...
        pipe: Option<Arc<Mutex<VecDeque<u8>>>>, // 用VecDeque作为缓冲区
    }

    impl File {
        ...
        // 设置当前pipe相关的fd属性，读/写，缓冲区指针
        pub fn set_pipe(&mut self, readable: bool, pipe: Arc<Mutex<VecDeque<u8>>>) {...}
        // 从缓冲区中读出一个字符
        pub fn pipe_pop(&mut self) -> Option<u8> {...}
        // 向缓冲区写入一个字符
        pub fn pipe_push(&mut self, c: u8) {...}
        // 若pipe的Arc::strong_count为1，说明另一端fd已析构，即为管道关闭
        pub fn pipe_is_on(&self) -> bool {...}
        // 缓冲区是否为空
        pub fn pipe_is_empty(&self) -> bool {...}
    }
   ```
   其中`pipe_pop, pipe_push, pipe_is_empty`就是VecDeque同名方法的简单封装。
3. `Thread`增加创建管道的接口`pipe`。
   ```rust
   impl Thread {
       // 创建pipe：申请两个新fd，创建空的pipe缓冲区，设置fd的读写属性和缓冲区指针
       pub fn pipe(&mut self) -> isize {
           let reader = self.alloc_fd() as isize;
           let writer = self.alloc_fd() as isize;
           let pipe = Arc::new(Mutex::new(VecDeque::default()));
           self.ofile[reader as usize].as_mut().unwrap().lock().set_pipe(true, pipe.clone());
           self.ofile[writer as usize].as_mut().unwrap().lock().set_pipe(false, pipe);
           (reader << 32) | writer
       }
   }
   ```
4. 扩展`sys_read, sys_write`以支持管道。
   ```rust
    unsafe fn sys_read(fd: usize, base: *mut u8, len: usize) -> isize {
        ...
        FileDescriptorType::FD_PIPE => {
            // 如果管道已关闭且缓冲区为空，直接返回
            if !file.pipe_is_on() && file.pipe_is_empty() { return 0; }
            loop { // 否则进入轮询，读到字符则返回，读不到字符则放弃当前时间片等待调度
                if let Some(c) = file.pipe_pop() { *base = c; break 1; }
                else { process::yield_now() }
            }
        }
        ...
    }
   ```
   注意上面`sys_read`中所用的`yield_now`是lab7中修改过的，只是放弃当前时间片等待调度，并不会进入睡眠。这样也就不需要设置`wake_up`的回调函数了。
   ```rust
    unsafe fn sys_write(fd: usize, base: *const u8, len: usize) -> isize {
        ... // 如果管道已关闭直接返回，否则循环写入所有字符
        FileDescriptorType::FD_PIPE => {
            if !file.pipe_is_on() { return 0; }
            let p = base;
            for _ in 0..len { file.pipe_push(*p); p.add(1); }
            return len as isize;
        }
        ...
    }
   ```


可能运行输出如下所示：
```
++++ setup timer!     ++++
fd_read = 3, fd_write = 4
message received in child process = Hello world! 
message sent to child process pid 1!
thread 0 exited, exit code = 0
thread 1 exited, exit code = 0
QEMU: Terminated
```
注意到上图中`Hello world!`之后出现了一个奇怪的字符，一开始我以为是`pipe`实现出bug了，返回了未经初始化的内容。后来花费了一些时间debug，才发现是评测脚本把最后一个`\0`也push到了字符串中，导致文本里面这里的最后一个字符是`\0`，在一般的文本编辑器中查看就出现了一个奇怪符号。用vim打开查看这一行：
```
message received in child process = Hello world!^@^M
```
可见这个位置的字符是`^@`，就是`\0`。
pipe实现圆满成功！rcore-lab完结撒花！:sparkles::sparkles::sparkles:

#### 思考题
1. 如果父进程还没写数据，子进程就开始读数据会怎么样？应如何解决？
   子进程读pipe时是不断轮询直到有内容，若无内容直接放弃当前时间片，等待下一次调度。

2. 简要说明你是如何保证读者和写者对于管道 Pipe 的访问不会触发 race condition 的？
   使用`Mutex`封装一下`VecDeque`，就可以避免race condition。

3. 在实现中是否曾遇到死锁？如果是，你是如何解决它的？
   遇到过，是读pipe时只是轮询，没有yield导致的。
