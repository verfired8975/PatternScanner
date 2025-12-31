use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Pattern {
    bytes: Vec<Option<u8>>,
    original: String,
}

impl Pattern {
    pub fn from_ida(pattern: &str) -> Result<Self> {
        let original = pattern.to_string();
        let mut bytes = Vec::new();

        for token in pattern.split_whitespace() {
            if token == "?" || token == "??" {
                bytes.push(None);
            } else {
                let byte = u8::from_str_radix(token, 16)
                    .map_err(|_| Error::InvalidPattern(format!("bu ne hex mi: {}", token)))?;
                bytes.push(Some(byte));
            }
        }

        if bytes.is_empty() {
            return Err(Error::InvalidPattern("bos pattern mi olur amk".to_string()));
        }

        Ok(Self { bytes, original })
    }

    pub fn from_code(bytes: &[u8], mask: &str) -> Result<Self> {
        if bytes.len() != mask.len() {
            return Err(Error::InvalidPattern("bytes ile mask uyusmuyor".to_string()));
        }

        let original = format!("code-style ({} bytes)", bytes.len());
        let pattern_bytes: Vec<Option<u8>> = bytes
            .iter()
            .zip(mask.chars())
            .map(|(&byte, mask_char)| {
                if mask_char == 'x' { Some(byte) } else { None }
            })
            .collect();

        Ok(Self { bytes: pattern_bytes, original })
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.iter().map(|&b| Some(b)).collect(),
            original: format!("raw ({} bytes)", bytes.len()),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub fn matches(&self, data: &[u8], offset: usize) -> bool {
        if offset + self.bytes.len() > data.len() {
            return false;
        }

        self.bytes.iter().enumerate().all(|(i, pattern_byte)| {
            match pattern_byte {
                Some(b) => data[offset + i] == *b,
                None => true,
            }
        })
    }

    pub fn first_byte(&self) -> Option<u8> {
        self.bytes.iter().find_map(|&b| b)
    }

    pub fn as_str(&self) -> &str {
        &self.original
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}
