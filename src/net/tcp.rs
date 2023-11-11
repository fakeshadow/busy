use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use crate::codec::Codec;

use super::MTU_DEFAULT;

/// 为Tcp绝对乐观分片的类型。
/// [FramedTcp::next]会尝试生产新的分片，[FramedTcp::send]会编码分片并发送。
/// 因任何问题导致的tcp流分包都会导致debug panic，因此该类型只适用于高速局域网内的低密度分片(分片一般要小于mtu)
pub struct FramedTcp<C, const LIMIT: usize = MTU_DEFAULT> {
    stream: TcpStream,
    buf: Vec<u8>,
    codec: C,
}

impl<C, const LIMIT: usize> FramedTcp<C, LIMIT> {
    pub fn new(stream: TcpStream, codec: C) -> Self {
        stream.set_nodelay(true).unwrap();
        stream.set_nonblocking(true).unwrap();
        Self {
            stream,
            buf: vec![0; LIMIT],
            codec,
        }
    }
}

impl<C, const LIMIT: usize> FramedTcp<C, LIMIT>
where
    C: Codec,
{
    /// 阻塞或中断返回None
    /// 读取错误会返回Some(io::Error)
    /// 对方关闭连接会返回Some(io::Error) io::ErrorKind为ConnectionAborted
    /// 成功接收消息返回Some(DecodeMsg)
    ///
    /// # Panics:
    /// tcp流读取字节数不小于const LIMIT会导致debug panic
    pub fn try_recv(&mut self) -> Option<io::Result<C::DecodeMsg>> {
        match self.stream.read(&mut self.buf) {
            Ok(0) => Some(Err(From::from(io::ErrorKind::ConnectionAborted))),
            Ok(n) => {
                debug_assert!(n < LIMIT, "stream message beyond limit");
                Some(self.codec.decode(&self.buf[..n]))
            }
            Err(ref e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                None
            }
            Err(e) => Some(Err(e)),
        }
    }

    /// 任何写入错误会返回io::Error，包括阻塞或中断
    /// 对方关闭连接会返回io::Error io::ErrorKind为ConnectionAborted
    ///     
    /// # Panics:
    /// tcp流无法一次发送编码完成的全部字节会导致debug panic
    pub fn send(&mut self, msg: C::EncodeMsg) -> io::Result<()> {
        let n = self.codec.encode(msg, &mut self.buf);
        match self.stream.write(&self.buf[..n])? {
            0 => Err(From::from(io::ErrorKind::ConnectionAborted)),
            n2 => {
                debug_assert_eq!(n, n2, "stream write failed with partial bytes succeed.");
                Ok(())
            }
        }
    }
}
