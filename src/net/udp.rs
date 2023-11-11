use std::{io, net::UdpSocket};

use crate::codec::Codec;

use super::MTU_DEFAULT;

/// 为Udp绝对乐观分片的类型。
/// [FramedUdp::next]会尝试生产新的分片，调用[FramedUdp::send]会编码分片并发送。
/// 因任何问题导致的udp消息分包都会导致debug panic，因此该类型只适用于高速局域网内的低密度分片(分片一般要小于mtu)
pub struct FramedUdp<C, const LIMIT: usize = MTU_DEFAULT> {
    socket: UdpSocket,
    buf: Vec<u8>,
    codec: C,
}

impl<C, const LIMIT: usize> FramedUdp<C, LIMIT> {
    pub fn new(socket: UdpSocket, codec: C) -> Self {
        socket.set_nonblocking(true).unwrap();
        Self {
            socket,
            buf: vec![0; LIMIT],
            codec,
        }
    }
}

impl<C, const LIMIT: usize> FramedUdp<C, LIMIT>
where
    C: Codec,
{
    /// 阻塞或中断返回None
    /// 读取错误会返回Some(io::Error)
    /// 对方关闭连接会返回Some(io::Error) io::ErrorKind为ConnectionAborted
    /// 成功接收消息返回Some(DecodeMsg)
    ///
    /// # Panics:
    /// udp取字节数不小于const LIMIT会导致debug panic
    pub fn try_recv(&mut self) -> Option<io::Result<C::DecodeMsg>> {
        match self.socket.recv(&mut self.buf) {
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

    /// 线程阻塞或中断会返回io::Error
    /// 对方关闭连接会返回io::Error io::ErrorKind为ConnectionAborted
    ///     
    /// # Panics:
    /// udp无法一次发送编码完成的全部字节会导致debug panic
    pub fn send(&mut self, msg: C::EncodeMsg) -> io::Result<()> {
        let n = self.codec.encode(msg, &mut self.buf);
        match self.socket.send(&self.buf[..n])? {
            0 => Err(From::from(io::ErrorKind::ConnectionAborted)),
            n2 => {
                debug_assert_eq!(n, n2, "socket send failed with partial bytes succeed.");
                Ok(())
            }
        }
    }
}
