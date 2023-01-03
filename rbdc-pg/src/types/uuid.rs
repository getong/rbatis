use std::str::FromStr;
use crate::arguments::PgArgumentBuffer;
use crate::types::decode::Decode;
use crate::types::encode::{Encode, IsNull};
use crate::value::{PgValue, PgValueFormat};
use rbdc::uuid::Uuid;
use rbdc::Error;

impl Encode for Uuid {
    fn encode(self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Error> {
        let uuid = uuid::Uuid::from_str(&self.0).map_err(|e|Error::from(e.to_string()))?;
        buf.extend_from_slice(uuid.as_bytes());
        Ok(IsNull::No)
    }
}

impl Decode for Uuid {
    fn decode(value: PgValue) -> Result<Self, Error> {
        Ok(Self(match value.format() {
            PgValueFormat::Binary => String::from_utf8(value.as_bytes()?.to_vec())
                .map_err(|e| Error::from(format!("Decode Uuid:{}", e)))?,
            PgValueFormat::Text => value
                .as_str()?
                .parse()
                .map_err(|e| Error::from(format!("Decode Uuid str:{}", e)))?,
        }))
    }
}
