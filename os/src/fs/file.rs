#![allow(non_camel_case_types)]
use alloc::sync::Arc;
use rcore_fs::vfs::INode;
use crate::fs::ROOT_INODE;
use spin::Mutex;
use alloc::collections::VecDeque;

#[derive(Copy,Clone,Debug)]
pub enum FileDescriptorType {
    FD_NONE,
    FD_INODE,
    FD_DEVICE,
    FD_PIPE,
}

#[derive(Clone)]
pub struct File {
    fdtype: FileDescriptorType,
    readable: bool,
    writable: bool,
    pub inode: Option<Arc<dyn INode>>,
    offset: usize,
    pipe: Option<Arc<Mutex<VecDeque<u8>>>>,
}

impl File {
    pub fn default() -> Self {
        File {
            fdtype: FileDescriptorType::FD_NONE,
            readable: false,
            writable: false,
            inode: None,
            offset: 0,
            pipe: None,
        }
    }
    pub fn set_readable(&mut self, v: bool) { self.readable = v; }
    pub fn set_writable(&mut self, v: bool) { self.writable = v; }
    pub fn get_readable(&self) -> bool { self.readable }
    pub fn get_writable(&self) -> bool { self.writable }
    pub fn set_fdtype(&mut self, t: FileDescriptorType) { self.fdtype = t; }
    pub fn get_fdtype(&self) -> FileDescriptorType { self.fdtype }
    pub fn set_offset(&mut self, o: usize) { self.offset = o; }
    pub fn get_offset(&self) -> usize { self.offset }

    pub fn open_file(&mut self, path: &'static str, flags: i32) {
        self.set_fdtype(FileDescriptorType::FD_INODE);
        self.set_readable(true);
        if (flags & 1) > 0 {
            self.set_readable(false);
        }
        if (flags & 3) > 0 {
            self.set_writable(true);
        }
        self.inode = Some(ROOT_INODE.lookup(path).unwrap().clone());
        self.set_offset(0);
    }

    pub fn set_pipe(&mut self, readable: bool, pipe: Arc<Mutex<VecDeque<u8>>>) {
        self.set_fdtype(FileDescriptorType::FD_PIPE);
        self.set_readable(readable);
        self.set_writable(!readable);
        self.pipe = Some(pipe);
    }

    pub fn pipe_pop(&mut self) -> Option<u8> {
        self.pipe.as_mut().unwrap().lock().pop_front()
    }

    pub fn pipe_push(&mut self, c: u8) {
        self.pipe.as_mut().unwrap().lock().push_back(c);
    }

    pub fn pipe_is_on(&self) -> bool {
        Arc::strong_count(self.pipe.as_ref().unwrap()) > 1
    }

    pub fn pipe_is_empty(&self) -> bool {
        self.pipe.as_ref().unwrap().lock().is_empty()
    }
}
