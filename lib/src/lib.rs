pub use anyhow::{
    anyhow,
    bail,
    Result
};

#[allow(unused_imports)]
pub(crate) use log::{
    debug,
    trace,
    info
};

pub(crate) use std::{
    fmt::{
	Display,
	Formatter,
    },
    io::{
	BufReader,
	Seek,
	SeekFrom,
	Read,
    }
};

pub use ndarray::{
    Array1,
    Array2,
    Array3
};

pub use tofas::{
    calendar::{
	GregorianDate,
	HMS,
	GregorianDateHMS
    },
    common::R,
};

pub mod cds_time;
pub mod consts;
pub mod grh;
pub mod giadr_l1c;
pub mod giadr_l2;
pub mod mdr_l1c;
pub mod mdr_l2;
pub mod mphr;
pub mod nat;
pub mod read_bin;
pub mod timestamp;
pub mod utils;

pub mod prelude {
    pub use super::*;

    pub use timestamp::ToUnix;
    pub use cds_time::ShortCdsTime;
    pub use giadr_l1c::*;
    pub use giadr_l2::*;
    pub use mdr_l1c::*;
    pub use mdr_l2::*;
    pub use mphr::*;
    pub use grh::*;
    pub use nat::*;
    pub use consts::*;
}

pub(crate) use timestamp::TimestampParser;
pub(crate) use read_bin::ReadBinBig;
pub(crate) use prelude::*;
pub(crate) use utils::*;
