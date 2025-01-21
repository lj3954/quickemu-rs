use std::{
    io::{Read, Write},
    net::TcpStream,
    os::unix::net::UnixStream,
};

use crate::{
    data::{Monitor, Network},
    error::MonitorError,
};

mod monitor;
