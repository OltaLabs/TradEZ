use rlp::Encodable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currencies {
    USDC,
    XTZ,
}

impl Encodable for Currencies {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match self {
            Currencies::USDC => s.append(&0u8),
            Currencies::XTZ => s.append(&1u8),
        };
    }
}

impl rlp::Decodable for Currencies {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value: u8 = rlp.as_val()?;
        match value {
            0 => Ok(Currencies::USDC),
            1 => Ok(Currencies::XTZ),
            _ => Err(rlp::DecoderError::Custom("Invalid Currencies value")),
        }
    }
}