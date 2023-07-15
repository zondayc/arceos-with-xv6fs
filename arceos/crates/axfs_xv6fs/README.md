# xv6fs相关内容介绍

## 添加&修改内容

arceos/crates/axfs_xv6fs/* \\
arceos/modules/axfs/src/sleeplock_shim.rs \\
arceos/modules/axfs/src/root.rs \\
arceos/modules/axfs/src/fs/axxv6fs.rs \\
arceos/modules/axfs/src/Cargo.toml \\
arceos/apps/fs/hellofs \\
xv6fs/*

## 支持功能

支持绝对路径的目录创建，不支持目录的递归创建和相对路径的创建
支持目录的递归删除
支持文件的读写创建和删除操作
文件系统自身还支持一系列系统调用，在xv6fs/file.rs中可以看到支持的系统调用功能

## 代码结构

axfs_xv6fs是arceos与xv6fs的接口层，里面实现了文件和目录的系统调用
axxv6fs负责在arceos中进行文件系统的初始化
xv6fs中提供给arceos的接口都在file.rs中，file.rs会调用下层的一系列功能
xv6fs-fuse是单独对文件系统的简单测试
xv6mkfs是生成文件系统磁盘镜像的相关代码

## PS
disk.img的生成在xv6mkfs中cargo run即可生成对应的disk.img，随后移动到arceos目录下即可。
