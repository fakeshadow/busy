use std::io;

/// 简易分片trait
pub trait Codec {
    /// 解码消息类型
    type DecodeMsg;

    /// 编码消息类型
    type EncodeMsg;

    /// 解码方法
    /// 完成解码后返回DecodeMsg。
    /// 解码错误时可以返回任意io::Error，该错误实体可以从FramedXXX::recev方法取回
    fn decode(&mut self, buf: &[u8]) -> io::Result<Self::DecodeMsg>;

    /// 编码方法，返回的usize必须等于编码完成后写入buf的字节总数。
    fn encode(&mut self, msg: Self::EncodeMsg, buf: &mut [u8]) -> usize;
}
