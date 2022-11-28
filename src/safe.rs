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
//!

use super::structs::UNIX_PATH_LEN;
use super::{
    AddressFamily, BindFamily, Inet6SockAddr, InetSockAddr, IpProto, SocketType, UnixSockAddr,
};
use std::ffi::{c_int, c_uint, c_ushort, c_void};
use std::mem::size_of;

extern "C" {
    fn socket(af: c_int, socktype: c_int, proto: c_int) -> c_int;
    fn bind(fd: c_int, s: *const c_void, slen: c_uint) -> c_int;
    fn connect(fd: c_int, s: *const c_void, slen: c_uint) -> c_int;
    fn htons(val: c_ushort) -> c_ushort;
    fn htonl(val: c_uint) -> c_uint;
    fn close(fd: c_int);
}

pub(super) fn safe_socket(af: AddressFamily, st: SocketType, pt: IpProto) -> i32 {
    unsafe { socket(af as c_int, st as c_int, pt as c_int) as i32 }
}

pub(super) fn safe_bind(fd: c_int, bf: BindFamily) -> i32 {
    match bf {
        BindFamily::Inet(addr, port) => bind_inet(fd, addr, port),
        BindFamily::Inet6(addr, port) => bind_inet6(fd, addr, port),
        BindFamily::Unix(path) => bind_unix(fd, path),
    }
}

pub(super) fn bind_inet(fd: c_int, ipaddr: u32, port: u16) -> i32 {
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

pub(super) fn bind_inet6(fd: c_int, ipaddr: u128, port: u16) -> i32 {
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

pub(super) fn bind_unix(fd: c_int, path: String) -> i32 {
    let size = if path.len() < (UNIX_PATH_LEN - 1) {
        path.len()
    } else {
        UNIX_PATH_LEN - 1
    };
    let mut stpath = [0u8; 108];
    let mut bytes = path.bytes();
    for i in 0..size {
        stpath[i] = bytes.next().unwrap();
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

pub(super) fn safe_connect(fd: c_int, bf: BindFamily) -> i32 {
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

pub(super) fn safe_close(fd: c_int) {
    unsafe {
        close(fd);
    }
}
