use serde::ser::{Serialize, Serializer};

use super::models::LengthPrefixedString;

impl Serialize for LengthPrefixedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_bytes();
        let mut output = Vec::<u8>::with_capacity(bytes.len() + 1);

        output.push(bytes.len() as u8);
        output.extend_from_slice(bytes);

        serializer.serialize_bytes(&output)
    }
}
