use std::ops::{Deref, DerefMut};

use alloy_primitives::hex::FromHex;
use rlp::{Decodable, Encodable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Address(pub alloy_primitives::Address);

impl From<alloy_primitives::Address> for Address {
    fn from(addr: alloy_primitives::Address) -> Self {
        Address(addr)
    }
}

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Address(alloy_primitives::Address::from(bytes))
    }
}

impl FromHex for Address {
    type Error = alloy_primitives::hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let addr = alloy_primitives::Address::from_hex(hex)?;
        Ok(Address(addr))
    }
}

impl Deref for Address {
    type Target = alloy_primitives::Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Encodable for Address {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.append_internal(&self.0.0.as_slice());
    }
}

impl Decodable for Address {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let bytes: Vec<u8> = rlp.as_val()?;
        Ok(Address(alloy_primitives::Address::from_slice(&bytes)))
    }
}

impl Address {
    pub const ZERO: Address = Address(alloy_primitives::Address::ZERO);
}
