//! Super block operations
use core::ptr;
use core::mem::{self, MaybeUninit};
use core::sync::atomic::{AtomicBool, Ordering};

use crate::fs_const::{ FSMAGIC, IPB, BPB };
use crate::buffer_cache::{ BLOCK_CACHE_MANAGER, BufData };

pub static mut SUPER_BLOCK: SuperBlock = SuperBlock::uninit();

/// In-memory copy of superblock
#[derive(Debug)]
pub struct SuperBlock {
    data: MaybeUninit<RawSuperBlock>,
    initialized: AtomicBool,
}

unsafe impl Sync for SuperBlock {}

impl SuperBlock {
    const fn uninit() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Read and init the super block from disk into memory.
    /// SAFETY: it should only be called by the first regular process alone.
    pub unsafe fn init(&mut self, dev: u32) {
        debug_assert_eq!(mem::align_of::<BufData>() % mem::align_of::<RawSuperBlock>(), 0);
        if self.initialized.load(Ordering::Relaxed) {
            return
        }
        let buf = BLOCK_CACHE_MANAGER.bread(dev, 1);
        //info!("super block init end bread");
        ptr::copy_nonoverlapping(
            buf.raw_data() as *const RawSuperBlock,
            self.data.as_mut_ptr(),
            1,
        );
        //info!("check magic number");
        if self.data.as_ptr().as_ref().unwrap().magic != FSMAGIC {
            panic!("invalid file system magic num");
        }
        //info!("superblock init data {:?}",self.data.as_ptr().as_ref().unwrap());
        self.initialized.store(true, Ordering::SeqCst);
        drop(buf);

        #[cfg(feature = "verbose_init_info")]
        info!("super block data: {:?}", self.data.as_ptr().as_ref().unwrap());
    }

    /// Read the info of super block.
    fn read(&self) -> &RawSuperBlock {
        debug_assert!(self.initialized.load(Ordering::Relaxed));
        unsafe {
            self.data.as_ptr().as_ref().unwrap()
        }
    }

    /// Load the log info of super block.
    /// Return starting block and usable blocks for log.
    pub fn read_log(&self) -> (u32, u32) {
        let sb = self.read();
        (sb.logstart, sb.nlog)
    }

    /// The total count of blocks in the disk.
    pub fn size(&self) -> u32 {
        let sb = self.read();
        sb.size
    }

    /// The inodestart of blocks
    pub fn inodestart(&self) -> u32 {
        let sb = self.read();
        sb.inodestart
    }

    /// bmapstart
    pub fn bmapstart(&self) -> u32 {
        let sb = self.read();
        sb.bmapstart
    }

    /// inode numbers
    pub fn ninodes(&self) -> u32 {
        self.read().ninodes
    }

    /// Given an inode number. 
    /// Return the blockno of the block this inode resides. 
    /// Panic if the queryed inode out of range. 
    pub fn locate_inode(&self, inum: u32) -> u32 {
        let sb = self.read();
        if inum >= sb.ninodes {
            panic!("query inum {} larger than maximum inode nums {}", inum, sb.ninodes);
        }
        // info!("[Debug] inum: {}", inum);
        let blockno = (inum / (IPB as u32)) + sb.inodestart;
        // info!("[Debug] block number: {}", blockno);
        blockno
    }

    /// Given a block number in the disk. 
    /// Returns the relevant block number of the (controlling) bitmap block. 
    pub fn bitmap_blockno(&self, blockno: u32) -> u32 {
        let sb = self.read();
        (blockno / BPB as u32) + sb.bmapstart
    }

    
}

/// Raw super block describes the disk layout.
#[repr(C)]
#[derive(Debug)]
pub struct RawSuperBlock {
    pub magic: u32,      // Must be FSMAGIC
    pub size: u32,       // Size of file system image (blocks)
    pub nblocks: u32,    // Number of data blocks
    pub ninodes: u32,    // Number of inodes
    pub nlog: u32,       // Number of log blocks
    pub logstart: u32,   // Block number of first log block
    pub inodestart: u32, // Block number of first inode block
    pub bmapstart: u32,  // Block number of first free map block
}

impl RawSuperBlock {
    pub fn new()->Self{
        RawSuperBlock { magic: 0, size: 0, nblocks: 0, ninodes: 0, 
            nlog: 0, logstart: 0, inodestart: 0, bmapstart: 0 }
    }
}