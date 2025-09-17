use std::io::{self, Read};
use std::fs::File;

pub struct SafeTruncateReader<'a> {
    file: &'a File,
    expected_size: u64,
    bytes_read: u64,
}

impl<'a> SafeTruncateReader<'a> {
    pub fn new(file: &'a File) -> io::Result<Self> {
        let expected_size = file.metadata()?.len();
        Ok(Self {
            file,
            expected_size,
            bytes_read: 0,
        })
    }
}

impl Read for SafeTruncateReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.bytes_read >= self.expected_size {
            return Ok(0);
        }
        
        let remaining = (self.expected_size - self.bytes_read) as usize;
        let to_read = buf.len().min(remaining);
        
        match self.file.read(&mut buf[..to_read]) {
            Ok(0) if self.bytes_read < self.expected_size => {
                // File was truncated - pad with zeros to maintain valid tar
                let pad_amount = to_read.min(buf.len());
                for i in 0..pad_amount {
                    buf[i] = 0;
                }
                self.bytes_read += pad_amount as u64;
                Ok(pad_amount)
            }
            Ok(n) => {
                self.bytes_read += n as u64;
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
}