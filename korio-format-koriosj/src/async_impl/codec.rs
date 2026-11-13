use std::io;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio_util::bytes::{BufMut as _, BytesMut};
use tokio_util::codec::{AnyDelimiterCodec, AnyDelimiterCodecError, Decoder, Encoder};

/// An error occurred while encoding or decoding a [JSON Lines].
/// 
/// [JSON Lines]: https://jsonlines.org/
#[derive(thiserror::Error, Debug)]
pub enum JsonLinesCodecError {
    /// The maximum line length was exceeded.
    #[error("max line length exceeded")]
    MaxLineLengthExceeded,

    /// An IO error occurred.
    #[error("{_0}")]
    Io(#[from] #[source] io::Error),

    /// A JSON error occurred.
    #[error("{_0}")]
    Json(#[source] serde_json::Error),
}

impl JsonLinesCodecError {
    fn from_codec(a: AnyDelimiterCodecError) -> Self {
        match a {
            AnyDelimiterCodecError::MaxChunkLengthExceeded => Self::MaxLineLengthExceeded,
            AnyDelimiterCodecError::Io(err) => Self::Io(err),
        }
    }
}

impl From<serde_json::Error> for JsonLinesCodecError {
    fn from(err: serde_json::Error) -> Self {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => {
                Self::Io(err.into())
            }
            Category::Syntax | Category::Data | Category::Eof => {
                Self::Json(err)
            }
        }
    }
}

impl From<JsonLinesCodecError> for io::Error {
    fn from(j: JsonLinesCodecError) -> Self {
        match j {
            JsonLinesCodecError::MaxLineLengthExceeded => io::Error::new(io::ErrorKind::InvalidData, j),
            JsonLinesCodecError::Io(err) => err,
            JsonLinesCodecError::Json(err) => err.into(),
        }
    }
}

/// A codec for [JSON Lines].
/// 
/// [JSON Lines]: https://jsonlines.org/
#[derive(Debug)]
pub struct JsonLinesCodec<Item> {
    decoder: AnyDelimiterCodec,
    _phantom: PhantomData<Item>,
}

impl<Item> JsonLinesCodec<Item> {
    /// Returns a `JsonLinesCodec` for [JSON Lines] values.
    /// 
    /// # Note
    /// 
    /// The returned JsonLinesCodec will not have an upper bound on the length
    /// of a buffered line. See the documentation for [`new_with_max_length`] for
    /// information on why this could be a potential security risk.
    /// 
    /// [JSON Lines]: https://jsonlines.org/
    pub fn new() -> Self {
        Self {
            decoder: Default::default(),
            _phantom: Default::default(),
        }
    }

    /// Returns a `JsonLinesCodec` with a maximum line length limit.
    /// 
    /// When the limit is reached a `JsonLinesCodecError` is returned and
    /// subsequent calls to to `decode` will discard bytes and return `None`
    /// until a newline is encountered and it will then function as normally.
    /// 
    /// # Note
    /// 
    /// Setting a length limit is highly recommended for any `JsonLinesCodec`
    /// that is exposed to untrusted as otherwise an attacker could send input
    /// that would lead to unbounded memory consumption.
    pub fn new_with_max_length(max_length: usize) -> Self {
        Self {
            decoder: AnyDelimiterCodec::new_with_max_length(vec![b'\n'], vec![b'\n'], max_length),
            _phantom: Default::default(),
        }
    }

    /// Returns the maximum line length when decoding.
    pub fn max_length(&self) -> usize {
        self.decoder.max_length()
    }

    /// Returns a new `JsonLinesCodec` that deals with `NItem` values instead.
    /// 
    /// This preserves the old state and [`max_length`] of the codec and is
    /// primarily for use with [`Framed::map_codec`].
    pub fn into_item_codec<NItem>(self) -> JsonLinesCodec<NItem> {
        JsonLinesCodec { decoder: self.decoder, _phantom: PhantomData }
    }
}

impl<Item> Default for JsonLinesCodec<Item> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Item> Encoder<Item> for JsonLinesCodec<Item>
where
    Item: Serialize,
{
    type Error = JsonLinesCodecError;

    fn encode(&mut self, item: Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        serde_json::to_writer(dst.writer(), &item)?;
        dst.put_u8(b'\n');
        Ok(())
    }
}

impl<Item> Decoder for JsonLinesCodec<Item>
where
    Item: DeserializeOwned,
{
    type Item = Item;

    type Error = JsonLinesCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(line) = self.decoder.decode(src).map_err(JsonLinesCodecError::from_codec)? {
            let item = serde_json::from_slice(&line)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
    
    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(line) = self.decoder.decode_eof(src).map_err(JsonLinesCodecError::from_codec)? {
            let item = serde_json::from_slice(&line)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}