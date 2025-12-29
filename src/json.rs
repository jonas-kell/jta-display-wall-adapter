use tokio_util::bytes::{Bytes, BytesMut};
use tokio_util::codec::{AnyDelimiterCodec, Decoder, Encoder};

pub fn make_json_exchange_codec() -> DelimiterAsBytesMutCodec {
    DelimiterAsBytesMutCodec::new([0x1E])
}

pub struct DelimiterAsBytesMutCodec {
    inner: AnyDelimiterCodec,
}

impl DelimiterAsBytesMutCodec {
    pub fn new(delimiter: impl Into<Vec<u8>> + Clone) -> Self {
        Self {
            inner: AnyDelimiterCodec::new_with_max_length(
                delimiter.clone().into(),
                delimiter.into(),
                2 << 20,
            ),
        }
    }
}

impl Decoder for DelimiterAsBytesMutCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self
            .inner
            .decode(src)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        {
            Some(bytes) => {
                // Convert Bytes -> BytesMut (one allocation)
                Ok(Some(BytesMut::from(bytes.as_ref())))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<Bytes> for DelimiterAsBytesMutCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let s = std::str::from_utf8(&item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        self.inner
            .encode(s, dst) // &str implements AsRef<str>
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}
