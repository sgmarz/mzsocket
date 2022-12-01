//! mzsocket
//! BSD socket wrapper using the standard C library
//! Stephen Marz
//! 27-Nov-2022

//! Copyright (c) 2022 Stephen Marz
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:

//! The above copyright notice and this permission notice shall be included in
//! all copies or substantial portions of the Software.

//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//! THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
//! THE SOFTWARE.

#[repr(C)]
#[allow(dead_code)]
pub enum BindFamily {
    Unix(String),
    Inet(u32, u16),
    Inet6(u128, u16),
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum AddressFamily {
    Unspec = 0,
    Unix = 1,
    Inet = 2,
    Inet6 = 10,
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum SocketType {
    Stream = 1,
    DataGram = 2,
    Raw = 3,
    SeqPacket = 5,
    Packet = 10,
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum IpProto {
    Ip = 0,
    Icmp = 1,
    Igmp = 2,
    IpIp = 4,
    Tcp = 6,
    Udp = 17,
    Ipv6 = 41,
    Gre = 47,
    Esp = 50,
    Ah = 51,
}

pub union InetAddr {
    pub addr8: [u8; 4],
    pub addr16: [u16; 2],
    pub addr32: u32
}

impl InetAddr {
    pub const fn new_8(addr8: [u8; 4]) -> Self {
        Self {
            addr8
        }
    }
    pub const fn new_16(addr16: [u16; 2]) -> Self {
        Self {
            addr16
        }
    }
    pub const fn new_32(addr32: u32) -> Self {
        Self {
            addr32
        }
    }
    pub const fn new(addr32: u32) -> Self {
        Self::new_32(addr32)
    }
}

impl Default for InetAddr {
    fn default() -> Self {
        Self {
            addr32: 0
        }
    }
}

#[repr(C)]
pub struct InetSockAddr {
    pub family: u16,
    pub port: u16,
    pub addr: InetAddr,
    pub reserved: u64,
}
impl Default for InetSockAddr {
    fn default() -> Self {
        Self {
            family: AddressFamily::Inet as u16,
            port: 0,
            addr: InetAddr::new(0),
            reserved: 0
        }
    }
}

pub union Inet6Addr {
    pub addr8: [u8; 16],
    pub addr16: [u16; 8],
    pub addr32: [u32; 4],
    pub addr64: [u64; 2],
    pub addr128: u128
}

impl Inet6Addr {
    pub const fn new_8(addr8: [u8; 16]) -> Self {
        Self {
            addr8
        }
    }
    pub const fn new_16(addr16: [u16; 8]) -> Self {
        Self {
            addr16
        }
    }
    pub const fn new_32(addr32: [u32; 4]) -> Self {
        Self {
            addr32
        }
    }
    pub const fn new_64(addr64: [u64; 2]) -> Self {
        Self {
            addr64
        }
    }
    pub const fn new_128(addr128: u128) -> Self {
        Self {
            addr128
        }
    }
}

impl Default for Inet6Addr {
    fn default() -> Self {
        Self {
            addr128: 0
        }
    }
}

#[repr(C)]
pub struct Inet6SockAddr {
    pub family: u16,
    pub port: u16,
    pub flowinfo: u32,
    pub addr: Inet6Addr,
    pub scopeid: u32,
}

impl Default for Inet6SockAddr {
    fn default() -> Self {
        Self {
            family: AddressFamily::Inet6 as u16,
            port: 0,
            flowinfo: 0,
            addr: Inet6Addr::default(),
            scopeid: 0
        }
    }
}

pub const UNIX_PATH_LEN: usize = 108;
#[repr(C)]
pub struct UnixSockAddr {
    pub family: u16,
    pub path: [u8; UNIX_PATH_LEN],
}


impl Default for UnixSockAddr {
    fn default() -> Self {
        Self {
            family: AddressFamily::Unix as u16,
            path: [0u8; UNIX_PATH_LEN]
        }
    }
}

