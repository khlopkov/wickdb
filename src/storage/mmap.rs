use memmap::Mmap;

use crate::storage::RandomAccessFile;
use crate::{Error, Result};
use std::io;

/// Implementation of RandomAccessFile using mmap syscall
/// Find more: https://man7.org/linux/man-pages/man2/mmap.2.html
impl RandomAccessFile for Mmap {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        if buf.len() == 0 {
            return Ok(());
        }
        let offset = offset as usize;
        if (offset as usize) + buf.len() > self.len() {
            let io_err =
                io::Error::new(io::ErrorKind::UnexpectedEof, "failed to fill whole buffer");
            return Err(Error::IO(io_err));
        }
        buf.clone_from_slice(&self[offset..(offset) + buf.len()]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use memmap::MmapOptions;

    use crate::storage::RandomAccessFile;
    use std::fs::{remove_file, File as SysFile};
    use std::io::Write;

    #[test]
    fn test_read_exact_at() {
        let mut f = SysFile::create("test").unwrap();
        f.write_all("hello world".as_bytes()).unwrap();
        f.sync_all().unwrap();
        let tests = vec![
            (0, "hello world"),
            (0, ""),
            (1, "ello"),
            (4, "o world"),
            (10, "d"),
            (11, ""),
            (100, ""),
        ];
        let sys_file = SysFile::open("test").unwrap();
        let mmap_file = unsafe { MmapOptions::new().map(&sys_file).unwrap() };
        let mut buffer = vec![];
        for (offset, expect) in tests {
            buffer.resize(expect.as_bytes().len(), 0u8);
            mmap_file
                .read_exact_at(buffer.as_mut_slice(), offset)
                .unwrap();
            assert_eq!(buffer, Vec::from(String::from(expect)));
        }
        // EOF case
        buffer.resize(12, 0u8);
        mmap_file
            .read_exact_at(buffer.as_mut_slice(), 2)
            .expect_err("failed to fill whole buffer");
        remove_file("test").unwrap();
    }
}
