use crate::sleeplock_shim::FsLockList;
use alloc::sync::Arc;
use alloc::vec::Vec;
use axfs_vfs::VfsOps;
use axfs_xv6fs::dir::DirNode;
use driver_block::BlockDriverOps;
use crate::BlockDevice as axdevice;

use xv6fs::interface::{INTERFACE_MANAGER,InterfaceManager,FsInterface};
use xv6fs::xv6fs::Xv6FS;
use xv6fs::BlockDevice;

use lazy_init::LazyInit;
use spin::mutex::Mutex;
use spin::rwlock::RwLock;


pub struct Xv6FileSystem{
    inner: Xv6FS,
}

impl VfsOps for Xv6FileSystem {
    fn root_dir(&self) -> axfs_vfs::VfsNodeRef {
        let root_dir=self.inner.get_root_vfile();
        Arc::new(DirNode{dirnode:root_dir})
    }
}

static BLOCK_DEV:LazyInit<Mutex<axdevice>>=LazyInit::new();
fn init_block_dev(blk_devs: axdevice){
    BLOCK_DEV.init_by(Mutex::new(blk_devs));
}

pub struct DiskOps;

impl BlockDevice for DiskOps {
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) {
        let _=BLOCK_DEV.lock().read_block(_block_id as u64, _buf);
    }

    fn write_block(&self, _block_id: usize, _buf: &[u8]) {
        let _=BLOCK_DEV.lock().write_block(_block_id as u64, _buf);
    }
}

impl Xv6FileSystem{
    pub fn new()->Self{
        Self { inner: Xv6FS::new()}
    }

    pub fn init(&mut self,blk_dev:axdevice){
        let interface=InterfaceManager{interface:Arc::new(AxFsInterface::new())};
        INTERFACE_MANAGER.init_by(interface);
        info!("init block device");
        init_block_dev(blk_dev);
        info!("init xv6fs");
        unsafe{xv6fs::init(Arc::new(DiskOps), 0);}
    }
}

pub struct AxFsInterface{
    fs_lock_list:RwLock<FsLockList>,
}

impl AxFsInterface {
    pub fn new()->Self{
        Self{ fs_lock_list: RwLock::new(FsLockList { lock_list:Vec::new() }) }
    }
}


impl FsInterface for AxFsInterface{
    fn get_cur_dir_inode(&self)->Option<xv6fs::inode::Inode> {
        None
    }
    fn new_sleep_lock(&self)->usize {
        self.fs_lock_list.write().new_lock()
    }
    fn sleep_cur_proc(&self,index:usize) {
        //info!("index is {}",index);
        //info!("before, lock is {}",self.fs_lock_list.read().lock_list[index].flag.load(core::sync::atomic::Ordering::Acquire));
        self.fs_lock_list.read().lock_list[index].sleep_cur_task();
        //info!("after, lock is {}",self.fs_lock_list.read().lock_list[index].flag.load(core::sync::atomic::Ordering::Acquire));
    }
    fn wake_up_next_proc(&self,index:usize) {
        self.fs_lock_list.read().lock_list[index].wake_up_next_proc();
    }
    fn get_flag(&self,index:usize)->bool {
        self.fs_lock_list.read().lock_list[index].flag.load(core::sync::atomic::Ordering::Acquire)
    }
}