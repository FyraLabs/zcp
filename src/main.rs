use nix::{
    fcntl::{fallocate, posix_fadvise, FallocateFlags, PosixFadviseAdvice},
    ioctl_read,
    libc::O_DIRECT,
    sys::sendfile::sendfile64,
};
use std::{
    env, fs,
    os::{
        fd::AsRawFd,
        unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt},
    },
    process::exit,
};

const BLKGETSIZE64_CODE: u8 = 0x12; // Defined in linux/fs.h
const BLKGETSIZE64_SEQ: u8 = 114;
ioctl_read!(ioctl_blkgetsize64, BLKGETSIZE64_CODE, BLKGETSIZE64_SEQ, u64);

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("usage: zcp <source> <destination>");
        exit(1);
    }

    let source_path = &args[1];
    let destination_path = &args[2];

    let source_metadata = fs::metadata(source_path)?;
    let source_is_block = source_metadata.file_type().is_block_device();
    let destination_is_block = fs::metadata(destination_path)
        .map(|destination_metadata| destination_metadata.file_type().is_block_device())
        .unwrap_or(false);

    let source = fs::OpenOptions::new()
        .read(true)
        .custom_flags(if source_is_block { O_DIRECT } else { 0 })
        .open(&args[1])?;
    let destination = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .custom_flags(if destination_is_block { O_DIRECT } else { 0 })
        .open(&args[2])?;

    let source_size = if source_is_block {
        let mut cap = 0u64;
        let cap_ptr = &mut cap as *mut u64;

        unsafe { ioctl_blkgetsize64(source.as_raw_fd(), cap_ptr)? };

        cap as i64
    } else {
        source_metadata.size() as i64
    };

    posix_fadvise(
        source.as_raw_fd(),
        0,
        source_size,
        PosixFadviseAdvice::POSIX_FADV_WILLNEED,
    )?;
    posix_fadvise(
        source.as_raw_fd(),
        0,
        source_size,
        PosixFadviseAdvice::POSIX_FADV_SEQUENTIAL,
    )?;

    if !destination_is_block {
        fallocate(
            destination.as_raw_fd(),
            FallocateFlags::empty(),
            0,
            source_size,
        )?;
    }

    let mut offset: i64 = 0;

    while offset < source_size {
        let count = (source_size - offset) as usize;
        // TODO: investigate using copy_file_range when that's more useful
        sendfile64(&destination, &source, Some(&mut offset), count)?;
    }

    Ok(())
}
