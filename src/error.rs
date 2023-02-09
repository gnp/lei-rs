#![warn(missing_docs)]
//! # lei::error
//!
//! Error type for LEI parsing and building.

use std::error::Error;
use std::fmt::Formatter;
use std::fmt::{Debug, Display};

use bstr::ByteSlice;

/// All the ways parsing or building could fail.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq)]
pub enum LEIError {
    /// The input length is not exactly 20 bytes.
    InvalidLength {
        /// The length we found
        was: usize,
    },
    /// The _Payload_ length is not exactly 18 bytes (checked when building).
    InvalidPayloadLength {
        /// The length we found
        was: usize,
    },
    /// The _LOU ID_ length is not exactly 4 bytes (checked when building).
    InvalidLouIdLength {
        /// The length we found
        was: usize,
    },
    /// The _Entity ID_ length is not exactly 14 bytes (checked when building).
    InvalidEntityIdLength {
        /// The length we found
        was: usize,
    },
    /// The input _LOU ID_ is not 4 uppercase ASCII alphanumeric characters.
    InvalidLouId {
        /// The _LOU ID_ we found
        was: [u8; 4],
    },
    /// The input _Entity ID_ is not 14 uppercase ASCII alphanumeric characters.
    InvalidEntityId {
        /// The _Entity ID_ we found
        was: [u8; 14],
    },
    /// The input _Check Digits_ is not two ASCII decimal digit characters.
    InvalidCheckDigits {
        /// The _Check Digits_ we found
        was: [u8; 2],
    },
    /// The input _Check Digits_ is in a valid format, but has an incorrect value.
    IncorrectCheckDigits {
        /// The _Check Digits_ we found
        was: [u8; 2],
        /// The _Check Digits_ we expected
        expected: [u8; 2],
    },
}

impl Debug for LEIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LEIError::InvalidLength { was } => {
                write!(f, "InvalidLength {{ was: {was:?} }}")
            }
            LEIError::InvalidPayloadLength { was } => {
                write!(f, "InvalidPayloadLength {{ was: {was:?} }}")
            }
            LEIError::InvalidLouIdLength { was } => {
                write!(f, "InvalidLouIdLength {{ was: {was:?} }}")
            }
            LEIError::InvalidEntityIdLength { was } => {
                write!(f, "InvalidEntityIdLength {{ was: {was:?} }}")
            }
            LEIError::InvalidLouId { was } => match std::str::from_utf8(was) {
                Ok(s) => {
                    write!(f, "InvalidLouId {{ was: {s:?} }}")
                }
                Err(_) => {
                    write!(f, "InvalidLouId {{ was: (invalid UTF-8) {was:?} }}")
                }
            },
            LEIError::InvalidEntityId { was } => match std::str::from_utf8(was) {
                Ok(s) => {
                    write!(f, "InvalidEntityId {{ was: {s:?} }}")
                }
                Err(_) => {
                    write!(f, "InvalidEntityId {{ was: (invalid UTF-8) {was:?} }}")
                }
            },
            LEIError::InvalidCheckDigits { was } => match std::str::from_utf8(was) {
                Ok(s) => {
                    write!(f, "InvalidCheckDigits {{ was: {s:?} }}")
                }
                Err(_) => {
                    write!(f, "InvalidCheckDigits {{ was: (invalid UTF-8) {was:?} }}")
                }
            },
            LEIError::IncorrectCheckDigits { was, expected } => {
                let was_utf8 = unsafe { was.to_str_unchecked() }; // This is safe because we know it is ASCII
                let expected_utf8 = unsafe { expected.to_str_unchecked() }; // This is safe because we know it is ASCII

                write!(
                    f,
                    "IncorrectCheckDigits {{ was: {was_utf8:?}, expected: {expected_utf8:?} }}"
                )
            }
        }
    }
}

impl Display for LEIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LEIError::InvalidLength { was } => {
                write!(f, "invalid length {was} bytes when expecting 20")
            }
            LEIError::InvalidPayloadLength { was } => {
                write!(f, "invalid Payload length {was} bytes when expecting 18")
            }
            LEIError::InvalidLouIdLength { was } => {
                write!(f, "invalid LOU ID length {was} bytes when expecting 4")
            }
            LEIError::InvalidEntityIdLength { was } => {
                write!(f, "invalid Entity ID length {was} bytes when expecting 14")
            }
            LEIError::InvalidLouId { was } => match std::str::from_utf8(was) {
                Ok(s) => {
                    write!(
                        f,
                        "prefix {s:?} is not 4 uppercase ASCII alphanumeric characters"
                    )
                }
                Err(_) => {
                    write!(f,
                    "prefix (invalid UTF-8) {was:?} is not 4 uppercase ASCII alphanumeric characters"
                    )
                }
            },
            LEIError::InvalidEntityId { was } => match std::str::from_utf8(was) {
                Ok(s) => {
                    write!(
                        f,
                        "basic code {s:?} is not 14 uppercase ASCII alphanumeric characters"
                    )
                }
                Err(_) => {
                    write!(f,
                "basic code (invalid UTF-8) {was:?} is not 14 uppercase ASCII alphanumeric characters"
                    )
                }
            },
            LEIError::InvalidCheckDigits { was } => match std::str::from_utf8(was) {
                Ok(s) => {
                    write!(f, "check digits {s:?} is not two ASCII decimal digits")
                }
                Err(_) => {
                    write!(
                        f,
                        "check digits (invalid UTF-8) {was:?} is not two ASCII decimal digits"
                    )
                }
            },
            LEIError::IncorrectCheckDigits { was, expected } => {
                let was_utf8 = unsafe { was.to_str_unchecked() }; // This is safe because we know it is ASCII
                let expected_utf8 = unsafe { expected.to_str_unchecked() }; // This is safe because we know it is ASCII

                write!(
                    f,
                    "incorrect check digits {was_utf8:?} when expecting {expected_utf8:?}"
                )
            }
        }
    }
}

impl Error for LEIError {}
