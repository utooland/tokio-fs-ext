use std::{
    cmp,
    io::{self, Read, Write},
    mem::MaybeUninit,
};

use tokio::io::ReadBuf;

pub(crate) const DEFAULT_MAX_BUF_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct Buf {
    pub(crate) buf: Vec<u8>,
    pos: usize,
}

/// Repeats operations that are interrupted.
macro_rules! uninterruptibly {
    ($e:expr) => {{
        loop {
            match $e {
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                res => break res,
            }
        }
    }};
}

impl Buf {
    pub(crate) fn with_capacity(n: usize) -> Buf {
        Buf {
            buf: Vec::with_capacity(n),
            pos: 0,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub(crate) fn copy_to(&mut self, dst: &mut ReadBuf<'_>) -> usize {
        let n = cmp::min(self.len(), dst.remaining());
        dst.put_slice(&self.bytes()[..n]);
        self.pos += n;

        if self.pos == self.buf.len() {
            self.buf.truncate(0);
            self.pos = 0;
        }

        n
    }

    pub(crate) fn copy_from(&mut self, src: &[u8], max_buf_size: usize) -> usize {
        assert!(self.is_empty());

        let n = cmp::min(src.len(), max_buf_size);

        self.buf.extend_from_slice(&src[..n]);
        n
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    /// # Safety
    ///
    /// `rd` must not read from the buffer `read` is borrowing and must correctly
    /// report the length of the data written into the buffer.
    pub(crate) unsafe fn read_from<T: Read>(
        &mut self,
        rd: &mut T,
        max_buf_size: usize,
    ) -> io::Result<usize> {
        assert!(self.is_empty());
        self.buf.reserve(max_buf_size);

        let buf = &mut self.buf.spare_capacity_mut()[..max_buf_size];
        // SAFETY: The memory may be uninitialized, but `rd.read` will only write to the buffer.
        let buf = unsafe { &mut *(buf as *mut [MaybeUninit<u8>] as *mut [u8]) };
        let res = uninterruptibly!(rd.read(buf));

        if let Ok(n) = res {
            // SAFETY: the caller promises that `rd.read` initializes
            // a section of `buf` and correctly reports that length.
            // The `self.is_empty()` assertion verifies that `n`
            // equals the length of the `buf` capacity that was written
            // to (and that `buf` isn't being shrunk).
            unsafe { self.buf.set_len(n) }
        } else {
            self.buf.clear();
        }

        assert_eq!(self.pos, 0);

        res
    }

    pub(crate) fn write_to<T: Write>(&mut self, wr: &mut T) -> io::Result<()> {
        assert_eq!(self.pos, 0);

        // `write_all` already ignores interrupts
        let res = wr.write_all(&self.buf);
        self.buf.clear();
        res
    }
}

impl Buf {
    pub(crate) fn discard_read(&mut self) -> i64 {
        let ret = -(self.bytes().len() as i64);
        self.pos = 0;
        self.buf.truncate(0);
        ret
    }

    pub(crate) fn copy_from_bufs(
        &mut self,
        bufs: &[io::IoSlice<'_>],
        max_buf_size: usize,
    ) -> usize {
        assert!(self.is_empty());

        let mut rem = max_buf_size;
        for buf in bufs {
            if rem == 0 {
                break;
            }

            let len = buf.len().min(rem);
            self.buf.extend_from_slice(&buf[..len]);
            rem -= len;
        }

        max_buf_size - rem
    }
}
