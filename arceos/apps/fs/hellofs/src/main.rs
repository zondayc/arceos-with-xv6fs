#![no_std]
#![no_main]

#[macro_use]
extern crate libax;

#[no_mangle]
fn main(){
    let res=libax::fs::create_dir("/testdir");
    let res=libax::fs::create_dir("/testdir/testsubdir");
    let res=libax::fs::write("/testdir/testsubdir/testfile", "hello");
    let res=libax::fs::read("/testdir/testsubdir/testfile");
    let buf=res.as_ref().unwrap();
    println!("{:?}",buf);
    let res=libax::fs::remove_dir("/testdir");
    let res=libax::fs::write("/testfile", "helloworld");
    let res=libax::fs::write("/testfile1", "helloworld");
    let res=libax::fs::read("/testfile");
    let buf=res.as_ref().unwrap();
    println!("{:?}",buf);
    let res=libax::fs::write("/testfile2", "let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!let me see who doesn't play the Arknight!!!");
    let res=libax::fs::remove_file("/testfile2");
    let res=libax::fs::remove_file("/testfile1");
    let res=libax::fs::remove_file("/testfile");
}