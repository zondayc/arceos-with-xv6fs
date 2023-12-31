use crate::bitmap::inode_alloc;
use crate::disk_inode::{InodeType,DirEntry};
use crate::fs_const::{ BSIZE, MAXOPBLOCKS, DIRSIZ };
use crate::inode::{ICACHE,Inode, InodeData};
use super::stat::Stat;
use crate::log::{LOG_MANAGER};
use alloc::vec::Vec;
use alloc::string::String;
use axlog::{info, debug};
use core::mem::size_of;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum FileType {
    None = 0,
    Pipe = 1,
    File = 2,
    Directory=3,
    Device = 4,
}

#[derive(Clone)]
pub struct Device {

}

#[derive(Clone)]
pub struct File {

}

#[derive(Clone)]
pub enum FileInner {
    Device(Device),
    File(File)
    // Pipe(Pipe)
}

/// Virtual File, which can abstract struct to dispatch 
/// syscall to specific file.
#[derive(Clone, Debug)]
pub struct VFile {
    pub(crate) ftype: FileType,
    pub(crate) readable: bool,
    pub(crate) writeable: bool,
    pub(crate) inode: Option<Inode>,
    pub(crate) offset: u32
    // inner: FileInner
}

impl VFile {
    pub const fn init() -> Self {
        Self{
            ftype: FileType::None,
            readable: false,
            writeable: false,
            inode: None,
            offset: 0,
        }
    }

    pub fn get_size(&self)->usize{
        let node=self.inode.as_ref().unwrap();
        let guard=node.lock();
        let res=guard.dinode.size;
        drop(guard);
        res as usize
    }

    ///addr is destination address 
    pub fn vfile_read(
        &self, 
        addr: usize,
        offset: usize,
        len: usize
    ) -> Result<usize, &'static str> {
        let ret;
        if !self.vfile_readable() {
            panic!("File can't be read!")
        }

        match self.ftype {
            FileType::File|FileType::Directory => {
                let inode = self.inode.as_ref().unwrap();
                let mut inode_guard = inode.lock();
                debug!("offset is {}",offset);
                match inode_guard.read( addr, offset as u32, len as u32) {
                    Ok(size) => {
                        ret = size;
                        let offset = unsafe { &mut *(&self.offset as *const _ as *mut u32)};
                        *offset += ret as u32;
                        drop(inode_guard);
                        Ok(ret)
                    },
                    Err(err) => {
                        Err(err)
                    }
                }
            },

            _ => {
                panic!("Invalid file!")
            },
        }
    }

    /// Write to file f. 
    /// addr is a user virtual address
    /// addr is src address
    /// 不涉及append操作，这个另外实现，通过给inode添加size或者添加fd table来实现
    pub fn vfile_write(
        &self, 
        offset:u32,
        addr: usize, 
        len: usize
    ) -> Result<usize, &'static str> {
        let ret; 
        if !self.vfile_writeable() {
            panic!("file can't be written")
        }
        
        match self.ftype {
            FileType::File|FileType::Directory => {
                // write a few blocks at a time to avoid exceeding 
                // the maxinum log transaction size, including
                // inode, indirect block, allocation blocks, 
                // and 2 blocks of slop for non-aligned writes. 
                // this really belongs lower down, since inode write
                // might be writing a device like console. 
                let max = ((MAXOPBLOCKS -1 -1 -2) / 2) * BSIZE;
                let mut count  = 0;
                let mut offset =offset;
                while count < len {
                    let mut write_bytes = len - count;
                    if write_bytes > max { write_bytes = max; }
                    info!("[Xv6fs] vfile_write: write bytes is {}",write_bytes);

                    // start log
                    //LOG.begin_op();
                    let inode = self.inode.as_ref().unwrap();
                    let mut inode_guard = inode.lock();

                    // return err when failt to write
                    inode_guard.write(
                        addr + count, 
                        offset, 
                        write_bytes as u32
                    )?;

                    // release sleeplock
                    drop(inode_guard);
                    LOG_MANAGER.end_op();
                    // end log
                    //LOG.end_op();

                    // update loop data
                    // self.offset += write_bytes as u32;
                    offset+=write_bytes as u32;
                    count += write_bytes;
                    
                }
                ret = count;
                Ok(ret)
            },

            _ => {
                panic!("Invalid File Type!")
            }
        }

    }

    pub fn vfile_append(
        &self, 
        addr: usize, 
        len: usize
    ) -> Result<usize, &'static str> {
        let ret; 
        if !self.vfile_writeable() {
            panic!("file can't be written")
        }
        match self.ftype {
            FileType::File|FileType::Directory => {
                let inode = self.inode.as_ref().unwrap();
                let inode_guard = inode.lock();
                let max = ((MAXOPBLOCKS -1 -1 -2) / 2) * BSIZE;
                let mut count  = 0;
                let mut offset=inode_guard.dinode.size;
                drop(inode_guard);
                while count < len {
                    let mut write_bytes = len - count;
                    if write_bytes > max { write_bytes = max; }
                    info!("[Xv6fs] vfile_write: write bytes is {}",write_bytes);
                    let mut inode_guard = inode.lock();
                    inode_guard.write(
                        addr + count, 
                        offset, 
                        write_bytes as u32
                    )?;
                    drop(inode_guard);
                    LOG_MANAGER.end_op();
                    offset+=write_bytes as u32;
                    count += write_bytes;
                }
                ret = count;
                Ok(ret)
            },
            _ => {
                panic!("Invalid File Type!")
            }
        }

    }

    fn vfile_readable(&self) -> bool {
        self.readable
    }

    fn vfile_writeable(&self) -> bool {
        self.writeable
    }

    /// Get metadata about file f. 
    /// addr is a user virtual address, pointing to a struct stat. 
    pub fn vfile_stat(&self) -> Result<Stat, &'static str> {
        let mut stat: Stat = Stat::new();
        match self.ftype {
            FileType::File|FileType::Directory => {
                let inode = self.inode.as_ref().unwrap();
                
                #[cfg(feature = "debug")]
                info!("[Kernel] stat: inode index: {}, dev: {}, inum: {}", inode.index, inode.dev, inode.inum);

                let inode_guard = inode.lock();
                inode_guard.stat(&mut stat);
                drop(inode_guard);
                
                // info!(
                //     "[Kernel] stat: dev: {}, inum: {}, nlink: {}, size: {}, type: {:?}", 
                //     stat.dev, stat.inum, stat.nlink, stat.size, stat.itype
                // );
                Ok(stat)
            },  

            _ => {
                Err("")
            }
        }
    }

    pub fn vfile_is_dir(&self)->bool{
        if self.ftype==FileType::Directory{
            return true;
        }
        false
    }

    pub fn vfile_is_file(&self)->bool{
        if self.ftype==FileType::File{
            return true;
        }
        false
    }

    pub fn vfile_create_file(path:&str,readable:bool,writeable:bool)->Option<Self>{
        info!("vfile create file: path is {}",path);
        let inode=ICACHE.create(path.as_bytes(),crate::disk_inode::InodeType::File, 2, 1).unwrap();
        LOG_MANAGER.end_op();
        Some(Self { ftype: FileType::File, readable, writeable, inode:Some(inode), offset:0})
    }

    pub fn vfile_create_dir(path:&str,readable:bool,writeable:bool)->Option<Self>{
        info!("vfile create dir: path is {}",path);
        let inode=ICACHE.create(path.as_bytes(),crate::disk_inode::InodeType::Directory, 2, 1).unwrap();
        LOG_MANAGER.end_op();
        Some(Self { ftype: FileType::Directory, readable, writeable, inode:Some(inode), offset:0})
    }

    pub fn vfile_lookup(path:&str)->Option<Self>{
        info!("vfile lookup: path is {}",path);
        match ICACHE.look_up(path.as_bytes()){
            Ok(node)=>{
                let guard=node.lock();
                let ty=match guard.dinode.itype{
                    InodeType::Directory=>FileType::Directory,
                    _=>FileType::File,
                };
                drop(guard);
                Some(Self { ftype: ty, readable:true, writeable:true, inode:Some(node), offset:0})
            },
            Err(_)=>None,
        }
    }

    pub fn vfile_readdir(&self)->Option<Vec<String>>{
        info!("vfile read dir");
        if self.ftype!=FileType::Directory{
            panic!("this is not a directory!");
        }
        let mut inode_data=self.inode.as_ref().unwrap().lock();
        inode_data.ls()
    }

    pub fn vfile_remove(&self,path:&str){
        info!("vfile remove");
        let _=ICACHE.remove(path.as_bytes());
        LOG_MANAGER.end_op();
    }

    pub fn vfile_create_under_dir(&self,file_name:&str,itype:InodeType)->Self{
        info!("vfile create: path is {}",file_name);
        let self_inode=self.inode.as_ref().unwrap();
        let mut self_idata=self_inode.lock();
        let dev=self_inode.dev;
        let inum=inode_alloc(dev,itype);
        info!("vfile create: inum is {}",inum);
        let inode=ICACHE.get(dev, inum);
        let mut idata=inode.lock();
        idata.dinode.major=2;
        idata.dinode.minor=1;
        idata.dinode.nlink=1;
        idata.update();
        let mut ftype=FileType::File;
        if itype==InodeType::Directory{
            ftype=FileType::Directory;
            idata.dinode.nlink+=1;
            idata.update();
            let _=idata.dir_link(".".as_bytes(), inum);
            let _=idata.dir_link("..".as_bytes(), self_inode.inum);
        }
        self_idata.dir_link(file_name.as_bytes(), inode.inum).expect("parent inode fail to link");
        drop(idata);
        drop(self_idata);
        LOG_MANAGER.end_op();
        VFile { ftype, readable:true, writeable:true, inode:Some(inode), offset:0}
        
    }

    pub fn vfile_size(&self)->usize{
        let inode=self.inode.as_ref().unwrap();
        let idata=inode.lock();
        idata.dinode.size as usize
    }

    pub fn vfile_link(&self,src_path:&str,dir_path:&str){
        let inode=match ICACHE.namei(src_path.as_bytes()) {
            Some(cur)=>{
                cur
            },
            None=>{
                panic!("[Xv6fs] vfile_link: not find src path");
            }
        };
        let mut inode_guard=inode.lock();
        if inode_guard.dinode.itype == InodeType::Directory {
            panic!("[Xv6fs] vfile_link: cannot link directory");
        }
        inode_guard.dinode.nlink+=1;
        let mut name = [0u8; DIRSIZ];
        let parent=match ICACHE.namei_parent(&dir_path.as_bytes(), &mut name) {
            Some(cur)=>{
                cur
            },
            None => {
                panic!("[Xv6fs] vfile_link: not find dir path");
            }
        };
        let mut parent_guard=parent.lock();
        let _=parent_guard.dir_link(&name, inode.inum);
        inode_guard.update();
        parent_guard.update();
        drop(parent_guard);
        drop(inode_guard);
        LOG_MANAGER.end_op();
    }

    pub fn vfile_unlink(&self,path:&str){//目录没有删掉dir entry
        info!("[Xv6fs] vfile unlink: unlink {}",path);
        let mut name = [0u8; DIRSIZ];
        let parent=match ICACHE.namei_parent(&path.as_bytes(), &mut name) {
            Some(cur)=>cur,
            None=>panic!("[Xv6fs] vfile_unlink: not find path")
        };
        let mut parent_guard=parent.lock();
        let inode=match parent_guard.dir_lookup(&name) {
            Some(cur) => {
                cur
            },
            _ => {
                panic!("[Xv6fs] vfile_unlink: not find name");
            }
        };
        let mut inode_guard=inode.lock();
        if inode_guard.dinode.itype==InodeType::Directory{
            panic!("[Xv6fs] vfile_link: cannot unlink directory");
        }
        inode_guard.dinode.nlink-=1;
        info!("now disk inode nlink is {}",inode_guard.dinode.nlink);
        let flag=match inode_guard.dinode.nlink {
            0=>true,
            _=>false
        };
        inode_guard.update();
        drop(inode_guard);
        if flag{
            drop(parent_guard);
            self.vfile_remove(path);
        }else{
            let _=parent_guard.dir_unlink(&name);
            drop(parent_guard);
        }
        LOG_MANAGER.end_op();
    }

    pub fn vfile_rename(&self,path:&str,new_name:&str){
        InodeData::rename(path, new_name);
    }

    pub fn vfile_pass_dir(&self)->Option<Vec<(String,InodeType)>>{
        let mut inode_guard=self.inode.as_ref().unwrap().lock();
        let mut v=Vec::new();
        let de_size = size_of::<DirEntry>();
        let mut dir_entry = DirEntry::new();
        let dir_entry_ptr = &mut dir_entry as *mut _ as *mut u8;
        for offset in (0..inode_guard.dinode.size).step_by(de_size) {
            inode_guard.read(
                dir_entry_ptr as usize, 
                offset, 
                de_size as u32
            ).expect("Cannot read entry in this dir");
            if dir_entry.inum == 0 {
                continue;
            }
            // info!("dir_entry_name: {}, name: {}", String::from_utf8(dir_entry.name.to_vec()).unwrap(), String::from_utf8(name.to_vec()).unwrap());
            let name=String::from_utf8(dir_entry.name.to_vec()).unwrap();
            let itype=ICACHE.get_inum_type(inode_guard.dev,dir_entry.inum as u32);
            v.push((name,itype));

        }
        info!("xv6fs: vfile pass dir is {:?}",v);
        Some(v)
    }

    pub fn vfile_truncate(&self,size:u64)->usize{
        let mut inode_guard=self.inode.as_ref().unwrap().lock();
        let res=inode_guard.resize(self.inode.as_ref().unwrap(), size);
        LOG_MANAGER.end_op();
        res

    }

    // pub fn test_sleep_lock(){
    //     let mut counter=0;
    //     let mut i=1;
    //     let count_lock=Arc::new(SleepLock::new(counter,init_lock()));
    //     let i_lock=SleepLock::new(i,init_lock());
    //     static SUM:AtomicI32=AtomicI32::new(0);
    //     for i in 0..3{
    //         let clock=count_lock.clone();
    //         axtask::spawn(move||{
    //             let g1=clock.lock();
    //             axtask::yield_now();
    //             drop(g1);
    //             info!("==========hello i===========, {}",i);
    //             axtask::yield_now();
    //             info!("hello i, {}, sum is {}",i,SUM.load(core::sync::atomic::Ordering::Acquire));
    //             SUM.fetch_add(1, core::sync::atomic::Ordering::Release);
    //             let g1=clock.lock();
    //             info!("==========hello i===========, {}",i);
    //             axtask::yield_now();
    //             SUM.fetch_add(1, core::sync::atomic::Ordering::Release);
    //             drop(g1);
    //         });  
    //     }
    //     loop {
    //         if SUM.load(core::sync::atomic::Ordering::Acquire)==6{
    //             break;
    //         }
    //         axtask::yield_now();
    //     } 
    //     info!("end!, sum is {}",SUM.load(core::sync::atomic::Ordering::Acquire));
    // }
}

pub fn test_link_unlink(){
    let inode=ICACHE.get_root_dir();
    let idata=inode.lock();
    let ftype=FileType::Directory;
    drop(idata);
    let root=VFile { 
        ftype,
        readable:true, 
        writeable:true, 
        inode:Some(inode), 
        offset:0,
    };
    root.vfile_readdir().map(|x| {
        for file_name in x {
            info!("{}", file_name);
        }
    })
    .expect("can't read root directory");
    root.vfile_create_under_dir("test\0", InodeType::File);
    root.vfile_readdir().map(|x| {
        for file_name in x {
            info!("{}", file_name);
        }
    })
    .expect("can't read root directory");
    //root.vfile_remove("/test\0");
    root.vfile_link("/test\0", "/test1\0");
    let data="hello".as_bytes();
    let test1=VFile::vfile_create_file("/test1\0", true, true).unwrap();
    let _=test1.vfile_write(0,data.as_ptr() as usize, data.len());
    root.vfile_unlink("/test1\0");
    root.vfile_unlink("/test\0");
    root.vfile_readdir().map(|x| {
        for file_name in x {
            info!("{}", file_name);
        }
    })
    .expect("can't read root directory");


}



