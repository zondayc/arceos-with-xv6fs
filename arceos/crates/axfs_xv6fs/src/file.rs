use axfs_vfs::{impl_vfs_non_dir_default,VfsNodeAttr,VfsNodeOps,VfsResult};
use log::info;
use xv6fs::file::VFile;

pub struct FileNode{
    pub filenode: VFile,
}

impl VfsNodeOps for FileNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let size=self.filenode.get_size();
        Ok(VfsNodeAttr::new_file(size as u64,0))//这里的filesize也有问题，这感觉还不太好办，，，
    }

    fn truncate(&self, size: u64) -> VfsResult {
        self.filenode.vfile_truncate(size);
        Ok(())
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        info!("begin read, offset is {}",offset);
        VfsResult::Ok(self.filenode.vfile_read(buf.as_mut_ptr() as usize, offset as usize,buf.len()).unwrap())
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        VfsResult::Ok(self.filenode.vfile_write(offset as u32,buf.as_ptr() as usize,buf.len()).unwrap())
    }

    impl_vfs_non_dir_default! {}
}