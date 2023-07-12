use alloc::string::String;
use axfs_vfs::{impl_vfs_dir_default,VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};
use log::info;
use xv6fs::disk_inode::InodeType;
use xv6fs::file::VFile;
use alloc::sync::{Arc, Weak};
use spin::RwLock;
use axerrno::AxError;

use crate::file::FileNode;

pub struct DirNode{
    pub dirnode: VFile,
    //parentnode: VFile,
}

impl VfsNodeOps for DirNode{
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(512,0))
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        todo!();
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        let mut root:String=String::from("/")+path+&String::from("\0");
        let path=&root;
        info!("axfs xv6fs: lookup path is {}",path);
        match VFile::vfile_lookup(path){
            Some(vfile)=>{
                info!("axfs xv6fs lookup: find path {}",path);
                if(vfile.vfile_is_dir()){
                    Ok(Arc::new(DirNode{dirnode:vfile}))
                }else {
                    Ok(Arc::new(FileNode{filenode:vfile}))
                }
            },
            None =>{
                info!("axfs xv6fs lookup: not find path {}",path);
                Err(AxError::NotFound)
            },
        }
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let dir_vec=self.dirnode.vfile_pass_dir().unwrap();
        let mut dir=dir_vec.iter().skip(2);
        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    if let Some((name, node_type)) = dir.next() {
                        //*ent = VfsDirEntry::new(name, node.get_attr().unwrap().file_type());
                        if node_type==&InodeType::Directory{
                            *ent=VfsDirEntry::new(name, VfsNodeType::Dir);
                        }else{
                            *ent=VfsDirEntry::new(name, VfsNodeType::File);
                        }
                    } else {
                        return Ok(i);
                    }
                }
            }
        }
        Ok(dirents.len())   
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        let mut root:String=String::from("/")+path+&String::from("\0");
        let path=&root;
        log::info!("axfs_xv6fs: create path is {}",path);
        if ty==VfsNodeType::File{
            VFile::vfile_create_file(path, true, true);
        }else{
            VFile::vfile_create_dir(path, true, true);
        }
        Ok(())
    }

    fn remove(&self, path: &str) -> VfsResult {
        let mut root:String=String::from("/")+path+&String::from("\0");
        let path=&root;
        log::info!("axfs_xv6fs: remove path is {}",path);
        self.dirnode.vfile_remove(path);
        Ok(())
    }

    impl_vfs_dir_default! {}
}

impl DirNode {
    pub fn new()->Self{
        DirNode { dirnode: VFile::init() }
    }
}