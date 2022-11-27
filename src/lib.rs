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
//! 
use std::{
    ffi::{c_int, c_longlong, c_uchar, c_uint, c_ulonglong, c_ushort, c_void},
    mem::size_of,
};

extern "C" {
    fn socket(af: c_int, socktype: c_int, proto: c_int) -> c_int;
    fn bind(fd: c_int, s: *const c_void, slen: c_uint) -> c_int;
    fn connect(fd: c_int, s: *const c_void, slen: c_uint) -> c_int;
    fn listen(fd: c_int, backlog: c_int) -> c_int;
    fn accept(fd: c_int, s: *mut c_void, slen: *mut c_uint) -> c_int;
    fn read(fd: c_int, buffer: *mut c_uchar, buflen: c_ulonglong) -> c_longlong;
    fn write(fd: c_int, buffer: *const c_uchar, buflen: c_ulonglong) -> c_longlong;
    fn htons(val: c_ushort) -> c_ushort;
    fn htonl(val: c_uint) -> c_uint;
    fn fcntl(fd: c_int, cmd: c_int, val: c_int) -> c_int;
    fn close(fd: c_int);
}

pub struct Socket {
    fd: c_int,
}

impl Socket {
    pub fn new(family: AddressFamily, st: SocketType, proto: Option<IpProto>) -> Result<Self, i32> {
        let pr = proto.unwrap_or(IpProto::Ip);
        let ws = safe_socket(family, st, pr);
        if ws < 0 {
            Err(ws)
        } else {
            Ok(Self { fd: ws })
        }
    }

    pub fn bind(&mut self, bf: BindFamily) -> Result<(), i32> {
        let r = safe_bind(self.fd, bf);
        if r < 0 {
            Err(r)
        } else {
            Ok(())
        }
    }

    pub fn listen(&mut self, backlog: i32) -> i32 {
        unsafe {
            listen(self.fd, backlog)
        }
    }

    pub fn acceptinet(&mut self) -> Result<(Socket, InetSockAddr), i32> {
        let mut isaddr = InetSockAddr::default();
        let mut slen = 0u32;
        let ret = unsafe {
            accept(self.fd, &mut isaddr as *mut InetSockAddr as *mut c_void, &mut slen as *mut u32 as *mut c_uint)
        } as usize;
        if slen as usize != size_of::<InetSockAddr>() {
            Err(ret as i32)
        }
        else {
            Ok((Self {fd: ret as i32}, isaddr))
        }
    }

    pub fn acceptinet6(&mut self) -> Result<(Socket, Inet6SockAddr), i32> {
        let mut isaddr = Inet6SockAddr::default();
        let mut slen = 0u32;
        let ret = unsafe {
            accept(self.fd, &mut isaddr as *mut Inet6SockAddr as *mut c_void, &mut slen as *mut u32 as *mut c_uint)
        } as usize;
        if slen as usize != size_of::<Inet6SockAddr>() {
            Err(ret as i32)
        }
        else {
            Ok((Self {fd: ret as i32}, isaddr))
        }
    }

    pub fn acceptunix(&mut self) -> Result<(Socket, UnixSockAddr), i32> {
        let mut isaddr = UnixSockAddr::default();
        let mut slen = 0u32;
        let ret = unsafe {
            accept(self.fd, &mut isaddr as *mut UnixSockAddr as *mut c_void, &mut slen as *mut u32 as *mut c_uint)
        } as usize;
        if slen as usize != size_of::<UnixSockAddr>() {
            Err(ret as i32)
        }
        else {
            Ok((Self {fd: ret as i32}, isaddr))
        }
    }

    pub fn connect(&mut self, bf: BindFamily) -> Result<(), i32> {
        let r = safe_connect(self.fd, bf);
        if r < 0 {
            Err(r)
        } else {
            Ok(())
        }
    }

    pub fn read(&self, buffer: &mut [u8]) -> Result<i64, i64> {
        unsafe {
            let ret = read(self.fd, buffer.as_mut_ptr(), buffer.len() as u64);
            if ret < 0 {
                Err(ret as i64)
            } else {
                Ok(ret as i64)
            }
        }
    }

    pub fn write(&self, buffer: &[u8]) -> Result<i64, i64> {
        unsafe {
            let ret = write(self.fd, buffer.as_ptr(), buffer.len() as u64);
            if ret < 0 {
                Err(ret as i64)
            }
            else {
                Ok(ret as i64)
            }
        }
    }

    pub fn setblocking(&mut self, block: bool) {
        const F_GETFL: c_int = 3;
        const F_SETFL: c_int = 4;
        const O_NONBLOCK: c_int = 0o4000;
        unsafe {
            let flags = fcntl(self.fd, F_GETFL, 0);
            let flags = if block {
                flags & !O_NONBLOCK
            }
            else {
                flags | O_NONBLOCK
            };
            fcntl(self.fd, F_SETFL, flags);
        }
    }

    pub fn block(&mut self) {
        self.setblocking(true);
    }

    pub fn nonblock(&mut self) {
        self.setblocking(false);
    }

    pub fn close(&mut self) {
        safe_close(self.fd);
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.close();
    }
}

#[repr(C)]
#[allow(dead_code)]
pub enum BindFamily {
    Unix(String),
    Inet(u32, u16),
    Inet6(u128, u16),
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
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

#[repr(C)]
pub struct InetSockAddr {
    family: u16,
    port: u16,
    addr: u32,
    reserved: u64,
}
impl Default for InetSockAddr {
    fn default() -> Self {
        Self {
            family: AddressFamily::Inet as u16,
            port: 0,
            addr: 0,
            reserved: 0
        }
    }
}

#[repr(C)]
pub struct Inet6SockAddr {
    family: u16,
    port: u16,
    flowinfo: u32,
    addr: [u8; 16],
    scopeid: u32,
}

impl Default for Inet6SockAddr {
    fn default() -> Self {
        Self {
            family: AddressFamily::Inet6 as u16,
            port: 0,
            flowinfo: 0,
            addr: [0u8; 16],
            scopeid: 0
        }
    }
}

pub const UNIX_PATH_LEN: usize = 108;
#[repr(C)]
pub struct UnixSockAddr {
    family: u16,
    path: [u8; UNIX_PATH_LEN],
}


impl Default for UnixSockAddr {
    fn default() -> Self {
        Self {
            family: AddressFamily::Unix as u16,
            path: [0u8; 108]
        }
    }
}

fn safe_socket(af: AddressFamily, st: SocketType, pt: IpProto) -> i32 {
    unsafe { socket(af as c_int, st as c_int, pt as c_int) as i32 }
}

fn safe_bind(fd: c_int, bf: BindFamily) -> i32 {
    match bf {
        BindFamily::Inet(addr, port) => bind_inet(fd, addr, port),
        BindFamily::Inet6(addr, port) => bind_inet6(fd, addr, port),
        BindFamily::Unix(path) => bind_unix(fd, path),
    }
}

fn bind_inet(fd: c_int, ipaddr: u32, port: u16) -> i32 {
    unsafe {
        let isa = InetSockAddr {
            family: AddressFamily::Inet as u16,
            port: htons(port),
            addr: htonl(ipaddr),
            reserved: 0,
        };
        bind(
            fd,
            &isa as *const InetSockAddr as *const c_void,
            size_of::<InetSockAddr>() as c_uint,
        ) as i32
    }
}

fn bind_inet6(fd: c_int, ipaddr: u128, port: u16) -> i32 {
    let ips = [
        ((ipaddr >> 120) & 0xFF) as u8,
        ((ipaddr >> 112) & 0xFF) as u8,
        ((ipaddr >> 104) & 0xFF) as u8,
        ((ipaddr >> 96) & 0xFF) as u8,
        ((ipaddr >> 88) & 0xFF) as u8,
        ((ipaddr >> 80) & 0xFF) as u8,
        ((ipaddr >> 72) & 0xFF) as u8,
        ((ipaddr >> 64) & 0xFF) as u8,
        ((ipaddr >> 56) & 0xFF) as u8,
        ((ipaddr >> 48) & 0xFF) as u8,
        ((ipaddr >> 40) & 0xFF) as u8,
        ((ipaddr >> 32) & 0xFF) as u8,
        ((ipaddr >> 24) & 0xFF) as u8,
        ((ipaddr >> 16) & 0xFF) as u8,
        ((ipaddr >> 8) & 0xFF) as u8,
        (ipaddr & 0xFF) as u8,
    ];
    unsafe {
        let isa = Inet6SockAddr {
            family: AddressFamily::Inet6 as u16,
            port: htons(port),
            flowinfo: 0,
            addr: ips,
            scopeid: 0,
        };
        bind(
            fd,
            &isa as *const Inet6SockAddr as *const c_void,
            size_of::<Inet6SockAddr>() as c_uint,
        ) as i32
    }
}

fn bind_unix(fd: c_int, path: String) -> i32 {
    let size = if path.len() < (UNIX_PATH_LEN - 1) { path.len() } else { UNIX_PATH_LEN - 1 };
    let mut stpath = [0u8; 108];
    let mut bytes = path.bytes();
    for i in 0..size {
        stpath[i] = bytes.nth(i).unwrap();
    }
    stpath[size] = 0;

    unsafe {
        let usa = UnixSockAddr {
            family: AddressFamily::Unix as u16,
            path: stpath,
        };
        bind(
            fd,
            &usa as *const UnixSockAddr as *const c_void,
            size_of::<UnixSockAddr>() as c_uint,
        ) as i32
    }
}

fn safe_connect(fd: c_int, bf: BindFamily) -> i32 {
    unsafe {
        match bf {
            BindFamily::Inet(addr, port) => {
                let s = InetSockAddr {
                    family: AddressFamily::Inet as u16,
                    port: htons(port),
                    addr: htonl(addr),
                    reserved: 0,
                };
                connect(
                    fd as i32,
                    &s as *const InetSockAddr as *const c_void,
                    size_of::<InetSockAddr>() as c_uint,
                ) as i32
            }
            BindFamily::Inet6(ipaddr, port) => {
                let saddr = [
                    ((ipaddr >> 120) & 0xFF) as u8,
                    ((ipaddr >> 112) & 0xFF) as u8,
                    ((ipaddr >> 104) & 0xFF) as u8,
                    ((ipaddr >> 96) & 0xFF) as u8,
                    ((ipaddr >> 88) & 0xFF) as u8,
                    ((ipaddr >> 80) & 0xFF) as u8,
                    ((ipaddr >> 72) & 0xFF) as u8,
                    ((ipaddr >> 64) & 0xFF) as u8,
                    ((ipaddr >> 56) & 0xFF) as u8,
                    ((ipaddr >> 48) & 0xFF) as u8,
                    ((ipaddr >> 40) & 0xFF) as u8,
                    ((ipaddr >> 32) & 0xFF) as u8,
                    ((ipaddr >> 24) & 0xFF) as u8,
                    ((ipaddr >> 16) & 0xFF) as u8,
                    ((ipaddr >> 8) & 0xFF) as u8,
                    (ipaddr & 0xFF) as u8,
                ];
                let s = Inet6SockAddr {
                    family: AddressFamily::Inet6 as u16,
                    port: htons(port),
                    flowinfo: 0,
                    addr: saddr,
                    scopeid: 0,
                };
                connect(
                    fd as i32,
                    &s as *const Inet6SockAddr as *const c_void,
                    size_of::<Inet6SockAddr>() as c_uint,
                ) as i32
            }
            BindFamily::Unix(path) => {
                let size = if path.len() <= 107 { path.len() } else { 107 };
                let mut stpath = [0u8; 108];
                let mut bytes = path.bytes();
                for i in 0..size {
                    stpath[i] = bytes.nth(i).unwrap();
                }
                stpath[size] = 0;
                let s = UnixSockAddr {
                    family: AddressFamily::Unix as u16,
                    path: stpath,
                };
                connect(
                    fd as i32,
                    &s as *const UnixSockAddr as *const c_void,
                    size_of::<UnixSockAddr>() as c_uint,
                ) as i32
            }
        }
    }
}

fn safe_close(fd: c_int) {
    unsafe {
        close(fd);
    }
}

pub fn inet_addr(addr: &str) -> u32 {
    let s = addr.split('.');
    let mut k = s.map(|z| 
        z.parse::<u8>().unwrap()
    );

    (k.next().unwrap() as u32) << 24 |
    (k.next().unwrap() as u32) << 16 |
    (k.next().unwrap() as u32) << 8  |
    (k.next().unwrap() as u32)
}
