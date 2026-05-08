use bytes::{Buf, BytesMut};

use crate::error::Result;

pub(crate) mod cip;
pub(crate) mod encap;
pub(crate) mod values;

pub(crate) trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

pub(crate) trait Decode: Sized {
    fn decode(buf: &mut impl Buf) -> Result<Self>;
}

#[cfg(test)]
mod tests;
