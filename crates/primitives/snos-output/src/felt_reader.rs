use starknet_api::hash::StarkFelt;

/// Error that may occur while reading felt array
#[derive(Clone, Debug)]
pub enum FeltReaderError {
    /// Reader reached the end of felt array
    OutOfBounds,
    /// Reader failed to cast felt to the target type
    InvalidCast,
}

#[cfg(feature = "std")]
impl std::error::Error for FeltReaderError {}

impl core::fmt::Display for FeltReaderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FeltReaderError::OutOfBounds => write!(f, "Reader reached the end of felt array"),
            FeltReaderError::InvalidCast => write!(f, "Reader failed to cast felt to the target type"),
        }
    }
}

/// Analogue of std::io::Cursor but for field elements.
/// Allows to sequentially read felts and felt segments given a reference to a felt array.
///
/// Felt segment is sub sequence of field elements within the original felt array.
/// Felt segments can be recursively embedded.
/// Felt segment is preceded by its size also encoded as felt, not included in segment size.
pub struct FeltReader<'a> {
    data: &'a [StarkFelt],
    offset: usize,
}

impl<'a> FeltReader<'a> {
    pub fn new(data: &'a [StarkFelt]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn remaining_len(&self) -> usize {
        self.data.len() - self.offset
    }

    pub fn read(&mut self) -> Result<StarkFelt, FeltReaderError> {
        if self.offset >= self.data.len() {
            return Err(FeltReaderError::OutOfBounds);
        }

        let res = self.data[self.offset];
        self.offset += 1;

        Ok(res)
    }

    pub fn read_segment(&mut self) -> Result<&'a [StarkFelt], FeltReaderError> {
        let segment_size = TryInto::<u64>::try_into(self.read()?).map_err(|_| FeltReaderError::InvalidCast)? as usize;
        let start = self.offset;

        if start + segment_size > self.data.len() {
            return Err(FeltReaderError::OutOfBounds);
        }

        self.offset += segment_size;

        Ok(&self.data[start..start + segment_size])
    }
}
