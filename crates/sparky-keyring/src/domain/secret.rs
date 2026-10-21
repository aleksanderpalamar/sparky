use std::fmt;

use zeroize::Zeroizing;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret {
    pub value: Zeroizing<Vec<u8>>,
    pub content_type: String,
}

impl Secret {
    pub fn new(value: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self { value: Zeroizing::new(value), content_type: content_type.into() }
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Secret").field("content_type", &self.content_type).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::Secret;

    #[test]
    fn o_debug_nunca_mostra_o_segredo() {
        let shown = format!("{:?}", Secret::new(b"hunter2".to_vec(), "text/plain"));
        assert!(!shown.contains("hunter2"));
        assert!(shown.contains("text/plain"));
    }
}
