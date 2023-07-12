use core::{sync::atomic::{AtomicBool,Ordering}};
use alloc::vec::Vec;
use axtask::WaitQueue;

pub struct FsLockManager{
    wq:WaitQueue,
    pub flag:AtomicBool,
}

pub struct FsLockList{
    pub lock_list:Vec<FsLockManager>,
}

impl FsLockManager {
    pub fn sleep_cur_task(&self){
        //info!("lock!,and flag is {}",self.flag.load(Ordering::Acquire));
        self.wq.wait_until(||self.flag.load(Ordering::Acquire)==false);
        //info!("lock!,and flag is {}",self.flag.load(Ordering::Acquire));
        self.flag.store(true, Ordering::Release);
        //info!("lock!,and flag is {}",self.flag.load(Ordering::Acquire));
    }
    pub fn wake_up_next_proc(&self){
        self.flag.store(false, Ordering::Release);
        self.wq.notify_one(true);
    }
    pub fn new()->Self{
        Self { wq: WaitQueue::new(), flag: AtomicBool::new(false) }
    }
}

impl FsLockList {
    pub fn new_lock(&mut self)->usize{
        let index=self.lock_list.len();
        let lock=FsLockManager::new();
        self.lock_list.push(lock);
        //info!("push!");
        index
    }
}
