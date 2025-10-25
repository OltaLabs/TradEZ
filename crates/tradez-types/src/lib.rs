use rlp::{Decodable, Encodable};

pub mod address;
#[cfg(feature = "api")]
pub mod api;
pub mod error;
pub mod orderbook;
pub mod position;

#[derive(Debug, PartialEq, Eq)]
pub struct SignedInput<T>
where
    T: Encodable + Decodable + PartialEq + Eq
{
    pub message: T,
    pub signature: Vec<u8>,
}

impl <T> SignedInput<T>
where
    T: Encodable + Decodable + PartialEq + Eq
{
    pub fn new(message: T, signature: Vec<u8>) -> Self {
        SignedInput { message, signature }
    }
}

impl<T> Encodable for SignedInput<T>
where
    T: Encodable + Decodable + PartialEq + Eq
{
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(2);
        s.append(&self.message);
        s.append(&self.signature);
    }
}

impl<T> Decodable for SignedInput<T>
where
    T: Encodable + Decodable + PartialEq + Eq
{
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let message: T = rlp.val_at(0)?;
        let signature: Vec<u8> = rlp.val_at(1)?;
        Ok(SignedInput {
            message,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use rlp::Encodable;

    use crate::{position::APIOrder, SignedInput};

    #[test]
    fn test_signed_input_rlp() {
        let message = APIOrder::default();
        let signature = vec![1, 2, 3, 4];
        let signed_input = SignedInput::new(message, signature);

        let encoded = signed_input.rlp_bytes();
        let decoded: SignedInput<APIOrder> = rlp::decode(&encoded).unwrap();

        assert_eq!(signed_input, decoded);
    }
}
