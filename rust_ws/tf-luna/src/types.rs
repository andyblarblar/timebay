use crate::error::Error;

/// Raw control frame, could be request or response.
pub struct ControlFrameRaw<'a> {
    /// Should be const 0x5A
    head: u8,
    /// length of bytes from head to checksum
    len: u8,
    /// Control frame type
    id: u8,
    /// Optional data payload, depends on frame type
    payload: Option<&'a [u8]>,
    /// Lower 8 bytes of the sum from head to payload
    chksum: u8,
}

impl<'a> TryFrom<&'a [u8]> for ControlFrameRaw<'a> {
    type Error = Error;

    /// Parses a raw control frame from bytes.
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let head = *value.first().ok_or(Error::TooShort)?;
        if head != 0x5A {
            return Err(Error::InvalidHead);
        }

        let len = *value.get(1).ok_or(Error::TooShort)?;

        // We don't need to check for length after this
        if value.len() as u8 != len {
            return Err(Error::TooShort);
        }

        let id = value[2];

        // Grab payload if it exists
        let payload = if len > 4 {
            Some(&value[3..len as usize - 1])
        } else {
            None
        };

        let chksum = value[len as usize - 1];

        Ok(Self {
            head,
            len,
            id,
            payload,
            chksum,
        })
    }
}

/// Default Data frame format
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct NineByteCm {
    /// Distance in cm
    pub dist: u16,
    /// Signal strength, reliable when > 100
    pub amp: u16,
    /// Temp in C
    pub temp: u16,
}

impl TryFrom<&[u8]> for NineByteCm {
    type Error = Error;

    /// Parses a frame. You may omit the header bytes.
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 7 {
            return Err(Error::TooShort);
        }

        // We don't care about the header bytes, so just slice them out (also allows users to omit them)
        let value = &value[value.len() - 7..];

        let dist = u16::from_le_bytes(value[0..=1].try_into().unwrap());
        let amp = u16::from_le_bytes(value[2..=3].try_into().unwrap());
        let temp = u16::from_le_bytes(value[4..=5].try_into().unwrap());

        // TODO use this, we may need to enable first
        let chksum = value[6];

        Ok(Self { dist, amp, temp })
    }
}
