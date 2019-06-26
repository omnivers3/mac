#![feature(try_from)]

//! 'mac' provides a common structure for, surprisingly,
//! Mac Addresses across cooperating network libraries.

extern crate serde;

use std::convert::{ TryFrom };
use std::fmt;
use std::str::FromStr;

// #[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub type MacByteTuple = (u8, u8, u8, u8, u8, u8);

/// Represents an error which occurred whilst parsing a MAC address
#[derive(Copy, Debug, Eq, PartialEq, Clone)]
pub enum MacAddressErrors {
    /// The MAC address has too few / many components, eg. 00:11
    InvalidLength (usize),
    /// One of the components contains an invalid value, eg. 00:GG:22:33:44:55
    InvalidComponent,
}

impl fmt::Display for MacAddressErrors {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MacAddressErrors::InvalidLength(len) => write!(fmt, "Expected 6 components but found {:}", len),
            MacAddressErrors::InvalidComponent => write!(fmt, "Invalid component in a MAC address string"),
        }
    }
}

/// A MAC address used to identify a unique machine
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MacAddress {
    bytes: [u8; 6],
}

impl MacAddress {
    /// Construct a new, empty, MacAddress
    pub fn new() -> MacAddress {
        MacAddress {
            bytes: [0, 0, 0, 0, 0, 0],
        }
    }

    /// Create a MacAddress from a set of individual bytes
    pub fn from_bytes(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddress {
        MacAddress {
            bytes: [a, b, c, d, e, f],
        }
    }

    /// Create a MacAddress from a tuple of bytes
    pub fn from_byte_tuple(bytes: MacByteTuple) -> MacAddress {
        MacAddress {
            bytes: [bytes.0, bytes.1, bytes.2, bytes.3, bytes.4, bytes.5],
        }
    }

    /// Create a MacAddress from a byte array
    pub fn from_byte_array(bytes: [u8; 6]) -> MacAddress {
        MacAddress { bytes }
    }

    pub fn from_byte_slice(bytes: &[u8]) -> Result<MacAddress, MacAddressErrors> {
        let len = bytes.len();
        if len == 6 {
            let mut a: [u8; 6] = Default::default();
            a.copy_from_slice(&bytes[0..6]);
            Ok(a.into())
            // Ok(MacAddress::new(v[0], v[1], v[2], v[3], v[4], v[5]))
        } else {
            Err(MacAddressErrors::InvalidLength(len))
            // Err(E::invalid_length(bytes.len(), &self))
        }
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5]
        )
    }
}

// #[cfg(feature = "serde")]
impl Serialize for MacAddress {
    /// Serializes the MAC address.
    ///
    /// It serializes either to a string or its binary representation, depending on what the format
    /// prefers.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("{}", self))
        } else {
            serializer.serialize_bytes(&self.bytes)
        }
    }
}

// #[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MacAddress {
    /// Deserializes the MAC address.
    ///
    /// It deserializes it from either a byte array (of size 6) or a string. If the format is
    /// self-descriptive (like JSON or MessagePack), it auto-detects it. If not, it obeys the
    /// human-readable property of the deserializer.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MacAddressVisitor;

        impl<'de> de::Visitor<'de> for MacAddressVisitor {
            type Value = MacAddress;

            fn visit_str<E: de::Error>(self, value: &str) -> Result<MacAddress, E> {
                value.parse().map_err(|err| E::custom(&format!("{}", err)))
            }

            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<MacAddress, E> {
                MacAddress::from_byte_slice(v)
                    .map_err(|_| E::invalid_length(v.len(), &self))
                // match v.into() {
                //     Ok (result) => Ok (result),
                //     Err (err) => Err(E::invalid_length(v.len(), &self))
                // }
                // if v.len() == 6 {
                //     Ok(MacAddress::new(v[0], v[1], v[2], v[3], v[4], v[5]))
                // } else {
                //     Err(E::invalid_length(v.len(), &self))
                // }
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "either a string representation of a MAC address or 6-element byte array"
                )
            }
        }

        // Decide what hint to provide to the deserializer based on if it is human readable or not
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(MacAddressVisitor)
        } else {
            deserializer.deserialize_bytes(MacAddressVisitor)
        }
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl From<MacByteTuple> for MacAddress {
    fn from(target: MacByteTuple) -> Self {
        MacAddress::from_byte_tuple(target)
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(target: [u8; 6]) -> Self {
        MacAddress::from_byte_array(target)
    }
}

impl <'a> TryFrom<&'a [u8]> for MacAddress {
    type Error = MacAddressErrors;

    fn try_from(target: &'a [u8]) -> Result<MacAddress, MacAddressErrors> {
        MacAddress::from_byte_slice(target)
    }
}

impl FromStr for MacAddress {
    type Err = MacAddressErrors;

    fn from_str(s: &str) -> Result<MacAddress, MacAddressErrors> {
        let mut parts = [0u8; 6];
        let splits = s.split(':');
        let mut i = 0;
        for split in splits {
            if i == 6 {
                return Err(MacAddressErrors::InvalidLength(i + 1));
            }
            match u8::from_str_radix(split, 16) {
                Ok(b) if split.len() != 0 => parts[i] = b,
                _ => return Err(MacAddressErrors::InvalidComponent),
            }
            i += 1;
        }

        if i == 6 {
            Ok(parts.into())
        } else {
            Err(MacAddressErrors::InvalidLength(i))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_addr_from_str() {
        assert_eq!(
            "00:00:00:00:00:00".parse(),
            Ok(MacAddress {
                bytes: [0, 0, 0, 0, 0, 0]
            })
        );
        assert_eq!(
            "ff:ff:ff:ff:ff:ff".parse(),
            Ok(MacAddress {
                bytes: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
            })
        );
        assert_eq!(
            "12:34:56:78:90:ab".parse(),
            Ok(MacAddress {
                bytes: [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB]
            })
        );
        assert_eq!(
            "::::::".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidComponent)
        );
        assert_eq!(
            "0::::::".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidComponent)
        );
        assert_eq!(
            "::::0::".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidComponent)
        );
        assert_eq!(
            "12:34:56:78".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidLength(4))
        );
        assert_eq!(
            "12:34:56:78:".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidComponent)
        );
        assert_eq!(
            "12:34:56:78:90".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidLength(5))
        );
        assert_eq!(
            "12:34:56:78:90:".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidComponent)
        );
        assert_eq!(
            "12:34:56:78:90:00:00".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidLength(7))
        );
        assert_eq!(
            "xx:xx:xx:xx:xx:xx".parse::<MacAddress>(),
            Err(MacAddressErrors::InvalidComponent)
        );
    }

    #[test]
    fn mac_addr_from_bytes() {
        assert_eq!(
            format!("{}", MacAddress::from_bytes(0, 0, 0, 0, 0, 0)),
            "00:00:00:00:00:00"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress::from_bytes(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF)
            ),
            "ff:ff:ff:ff:ff:ff"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress::from_bytes(0x12, 0x34, 0x56, 0x78, 0x90, 0xAB)
            ),
            "12:34:56:78:90:ab"
        );
    }

    #[test]
    fn mac_addr_from_byte_tuple() {
        assert_eq!(
            format!("{}", MacAddress::from_byte_tuple((0, 0, 0, 0, 0, 0))),
            "00:00:00:00:00:00"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress::from_byte_tuple((0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF))
            ),
            "ff:ff:ff:ff:ff:ff"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress::from_byte_tuple((0x12, 0x34, 0x56, 0x78, 0x90, 0xAB))
            ),
            "12:34:56:78:90:ab"
        );
    }

    #[test]
    fn mac_addr_from_byte_array() {
        assert_eq!(
            format!("{}", MacAddress::from_byte_array([0, 0, 0, 0, 0, 0])),
            "00:00:00:00:00:00"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress::from_byte_array([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
            ),
            "ff:ff:ff:ff:ff:ff"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress::from_byte_array([0x12, 0x34, 0x56, 0x78, 0x90, 0xAB])
            ),
            "12:34:56:78:90:ab"
        );
    }

    #[test]
    fn str_from_mac_addr() {
        assert_eq!(
            format!(
                "{}",
                MacAddress {
                    bytes: [0, 0, 0, 0, 0, 0]
                }
            ),
            "00:00:00:00:00:00"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress {
                    bytes: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
                }
            ),
            "ff:ff:ff:ff:ff:ff"
        );
        assert_eq!(
            format!(
                "{}",
                MacAddress {
                    bytes: [0x12, 0x34, 0x56, 0x78, 0x09, 0xAB]
                }
            ),
            "12:34:56:78:09:ab"
        );
    }

    // #[cfg(feature = "serde")]
    mod serde {
        extern crate serde_test;
        use self::serde_test::{
            assert_de_tokens, assert_de_tokens_error, assert_tokens, Compact, Configure, Readable,
            Token,
        };
        use super::*;

        #[test]
        fn string() {
            let mac = MacAddress::from_bytes(0x11, 0x22, 0x33, 0x44, 0x55, 0x66);
            assert_tokens(&mac.readable(), &[Token::Str("11:22:33:44:55:66")]);
            assert_de_tokens(&mac.readable(), &[Token::String("11:22:33:44:55:66")]);
            assert_de_tokens(&mac.readable(), &[Token::BorrowedStr("11:22:33:44:55:66")]);
            assert_de_tokens_error::<Readable<MacAddress>>(
                &[Token::Str("not an address")],
                "Invalid component in a MAC address string",
            );
            // It still can detect bytes if provided
            assert_de_tokens(
                &mac.readable(),
                &[Token::Bytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66])],
            );
        }

        #[test]
        fn bytes() {
            let mac = MacAddress::from_bytes(0x11, 0x22, 0x33, 0x44, 0x55, 0x66);
            assert_tokens(
                &mac.compact(),
                &[Token::Bytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66])],
            );
            assert_de_tokens(
                &mac.compact(),
                &[Token::BorrowedBytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66])],
            );
            let err = "invalid length 2, expected either a string representation of a MAC address or 6-element byte array";
            assert_de_tokens_error::<Compact<MacAddress>>(&[Token::Bytes(&[0x11, 0x33])], err);
            let err = "invalid length 7, expected either a string representation of a MAC address or 6-element byte array";
            assert_de_tokens_error::<Compact<MacAddress>>(
                &[Token::Bytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77])],
                err,
            );
            // Still can decode string in the compact mode
            assert_de_tokens(&mac.compact(), &[Token::Str("11:22:33:44:55:66")]);
        }
    }

}
