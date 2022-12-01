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

mod safe;
mod structs;

use std::{
    ffi::{c_int, c_longlong, c_uchar, c_uint, c_ulonglong, c_void},
    mem::size_of,
};

extern "C" {
    fn listen(fd: c_int, backlog: c_int) -> c_int;
    fn accept(fd: c_int, s: *mut c_void, slen: *mut c_uint) -> c_int;
    fn read(fd: c_int, buffer: *mut c_uchar, buflen: c_ulonglong) -> c_longlong;
    fn write(fd: c_int, buffer: *const c_uchar, buflen: c_ulonglong) -> c_longlong;
    fn fcntl(fd: c_int, cmd: c_int, val: c_int) -> c_int;
}

// Re-exports

pub type AddressFamily = structs::AddressFamily;
pub type SocketType = structs::SocketType;
pub type IpProto = structs::IpProto;
pub type BindFamily = structs::BindFamily;
pub type InetSockAddr = structs::InetSockAddr;
pub type Inet6SockAddr = structs::Inet6SockAddr;
pub type UnixSockAddr = structs::UnixSockAddr;

pub struct Socket {
    fd: c_int,
}

impl Socket {
    pub fn new(family: AddressFamily, st: SocketType, proto: Option<IpProto>) -> Result<Self, i32> {
        let pr = proto.unwrap_or(IpProto::Ip);
        let ws = safe::safe_socket(family, st, pr);
        if ws < 0 {
            Err(ws)
        } else {
            Ok(Self { fd: ws })
        }
    }

    pub fn bind(&mut self, bf: BindFamily) -> Result<(), i32> {
        let r = safe::safe_bind(self.fd, bf);
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
        let r = safe::safe_connect(self.fd, bf);
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
        safe::safe_close(self.fd);
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.close();
    }
}





/// Convert an Internet version 4 address from a string
/// into a u32 address.
/// 
/// * Returns a `Result<u32, usize>`. If the result is Err, it will return
/// the first index dotted quad to fail. The first dotted quad
/// is 0, the second is 1, and so forth. If the result is Ok,
/// the wrapped value will be a u32 of the IP address.
/// 
/// * Unspecified values of an incomplete IP address are set to 0.
/// 
/// * Returns in host byte order.
/// 
/// # Examples
/// 
/// ```
/// // Usage with a full IPv4 address.
/// let addr = mzsocket::inet_addr("127.64.32.8").unwrap();
/// // prints 0x7f402008
/// println!("0x{:08x}", addr);
/// 
/// // Usage and result of an incomplete IPv4 address
/// let addr = mzsocket::inet_addr("127.64").unwrap();
/// // prints 0x7f400000
/// println!("0x{:08x}", addr);
/// 
/// // Usage and result of an unparseable IPv4 address
/// let addr = mzsocket::inet_addr("127.168.john.p");
/// // prints Error @ 2
/// println!("Error @ {}", addr.unwrap_err());
/// 
/// // Usage and result of an IPv4 address with invalid numbers
/// let addr = mzsocket::inet_addr("127.512.711.299");
/// // prints Error @ 1
/// println!("Error @ {}", addr.unwrap_err());
pub fn inet_addr(addr: &str) -> Result<u32, usize> {
    let mut ret = 0;
    let mut addrs = addr.split('.');
    for i in 0..4 {
        if let Some(k) = addrs.next() {
            let parsed_result = k.parse::<u32>();
            if parsed_result.is_err() {
                return Err(i)
            }
            let parsed_value = parsed_result.unwrap();
            if parsed_value > 255 {
                return Err(i);
            }
            let bitp = 8 * (3 - i);
            ret |= parsed_value << bitp;
        }
        else {
            return Ok(ret)
        }
    }
    Ok(ret)
}
