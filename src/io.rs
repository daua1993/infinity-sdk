use bytes::Buf;
use tokio_util::codec::{Decoder, Encoder};

use crate::Data;
use crate::error::EslError;

#[derive(Debug, Clone)]
pub(crate) struct EslCodec {}

impl Encoder<&[u8]> for EslCodec {
    type Error = EslError;
    fn encode(&mut self, item: &[u8], dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item);
        Ok(())
    }
}

impl Decoder for EslCodec {
    type Item = Data;
    type Error = EslError;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 6 {
            return Ok(None);
        }
        // read length
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;
        // read service
        let mut service_bytes = [0u8; 2];
        service_bytes.copy_from_slice(&src[4..6]);
        let service = u16::from_be_bytes(service_bytes);
        // check if we have enough data
        if src.len() < length + 6 {
            return Ok(None);
        }
        // read data
        let data = src[6..length + 6].to_vec();
        src.advance(length + 6);
        // convert data to string
        match String::from_utf8(data) {
            Ok(string) => Ok(Some(Data { service, data: string })),
            Err(utf8_error) => {
                Err(EslError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    utf8_error.utf8_error(),
                )))
            }
        }
    }
}
