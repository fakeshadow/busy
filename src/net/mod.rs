//! 乐观积极的网络类型

mod tcp;
mod udp;

pub use tcp::FramedTcp;
pub use udp::FramedUdp;

const MTU_DEFAULT: usize = 1600;
