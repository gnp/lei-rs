#![warn(missing_docs)]
//! # lei
//!
//! `lei` provides an `LEI` type for working with validated Legal Entity Identifiers (LEIs) as
//! defined in [ISO 17442-1:2020](https://www.iso.org/standard/78829.html) "Financial services
//! &mdash; Legal entity identifier (LEI) &mdash; Part 1: Assignment" ("The Standard").
//!
//! An LEI is comprised of 20 ASCII characters with the following parts, in order:
//!
//! 1. A four-character uppercase alphanumeric _LOU Identifier_ corresponding to the _Local
//! Operating Unit_ that issued the LEI, according to [the GLEIF web
//! site](https://www.gleif.org/en/about-lei/iso-17442-the-lei-code-structure).
//! 2. A 14-character uppercase alphanumeric _Entity Identifier_ assigned by the corresponding
//! LOU.
//! 3. Two decimal digits representing the _Check Digit Pair_ computed using a method based on the
//! ISO/IEC 7064, MOD 97-10 _Check Character System_.
//!
//! Use the `parse()` or `parse_loose()` methods on the LEI type to convert a string to a validated
//! LEI.
//!
//! ## Related crates
//!
//! This crate is part of the Financial Identifiers series:
//!
//! * [CUSIP](https://crates.io/crates/cusip): Committee on Uniform Security Identification Procedures (ANSI X9.6-2020)
//! * [ISIN](https://crates.io/crates/isin): International Securities Identification Number (ISO 6166:2021)
//! * [LEI](https://crates.io/crates/lei): Legal Entity Identifier (ISO 17442:2020)
//!
//! The referenced ISO/IEC 7064, MOD 97-10 _Check Character System_ is implemented in:
//!
//! * [ISO/IEC 7064](https://crates.io/crates/iso_iec_7064): Check character systems (ISO/IEC 7064:2003)

use std::fmt;
use std::str::FromStr;

use bstr::ByteSlice;

use iso_iec_7064::{System, MOD_97_10};

pub mod error;
pub use error::LEIError;

mod digits;

use digits::DigitsIterator;

/// Compute the _Check Digits_ for an array of u8. No attempt is made to ensure the input string
/// is in the LEI payload format or length. If an illegal character (not an ASCII digit and not
/// an ASCII uppercase letter) is encountered, this function will panic.
fn compute_check_digits(s: &[u8]) -> [u8; 2] {
    let it = DigitsIterator::new(s);

    match MOD_97_10.checksum_ascii_bytes_iter(it) {
        Some(sum) => {
            let d1 = b'0' + (sum / 10) as u8;
            let d0 = b'0' + (sum % 10) as u8;
            let r: [u8; 2] = [d1, d0];
            r
        }
        None => {
            panic!("MOD_97_10::checksum() failed to produce a checksum! Invalid input characters?")
        }
    }
}

fn validate_lou_id_format(li: &[u8]) -> Result<(), LEIError> {
    if li.len() != 4 {
        panic!("Expected 4 bytes for LOU ID, but got {}", li.len());
    }

    for b in li {
        if !(b.is_ascii_digit() || (b.is_ascii_alphabetic() && b.is_ascii_uppercase())) {
            let mut li_copy: [u8; 4] = [0; 4];
            li_copy.copy_from_slice(li);
            return Err(LEIError::InvalidLouId { was: li_copy });
        }
    }
    Ok(())
}

fn validate_entity_id_format(ei: &[u8]) -> Result<(), LEIError> {
    if ei.len() != 14 {
        panic!("Expected 14 bytes for Entity ID, but got {}", ei.len());
    }

    for b in ei {
        if !(b.is_ascii_digit() || (b.is_ascii_alphabetic() && b.is_ascii_uppercase())) {
            let mut ei_copy: [u8; 14] = [0; 14];
            ei_copy.copy_from_slice(ei);
            return Err(LEIError::InvalidEntityId { was: ei_copy });
        }
    }
    Ok(())
}

fn validate_check_digits_format(cd: &[u8]) -> Result<(), LEIError> {
    if cd.len() != 2 {
        panic!("Expected 2 bytes for Check Digits, but got {}", cd.len());
    }

    for b in cd {
        if !(b.is_ascii_digit()) {
            let mut cd_copy: [u8; 2] = [0; 2];
            cd_copy.copy_from_slice(cd);
            return Err(LEIError::InvalidCheckDigits { was: cd_copy });
        }
    }
    Ok(())
}

/// Parse a string to a valid LEI or an error message, requiring the string to already be only
/// uppercase alphanumerics with no leading or trailing whitespace in addition to being the
/// right length and format.
pub fn parse(value: &str) -> Result<LEI, LEIError> {
    let v: String = value.into();

    if v.len() != 20 {
        return Err(LEIError::InvalidLength { was: v.len() });
    }

    // We make the preliminary assumption that the string is pure ASCII, so we work with the
    // underlying bytes. If there is Unicode in the string, the bytes will be outside the
    // allowed range and format validations will fail.

    let b = v.as_bytes();

    // We slice out the three fields and validate their formats.

    let lou_id: &[u8] = &b[0..4];
    validate_lou_id_format(lou_id)?;

    let entity_id: &[u8] = &b[4..18];
    validate_entity_id_format(entity_id)?;

    let check_digits = &b[18..20];
    validate_check_digits_format(check_digits)?;

    // Now, we need to compute the correct check digit value from the "payload" (everything except
    // the check digit).

    let payload = &b[0..18];

    let computed_check_digits = compute_check_digits(payload);

    let incorrect_check_digits = check_digits != computed_check_digits;
    if incorrect_check_digits {
        let mut cd_copy: [u8; 2] = [0; 2];
        cd_copy.copy_from_slice(check_digits);
        return Err(LEIError::IncorrectCheckDigits {
            was: cd_copy,
            expected: computed_check_digits,
        });
    }

    let mut bb = [0u8; 20];
    bb.copy_from_slice(b);

    Ok(LEI(bb))
}

/// Parse a string to a valid LEI or an error, allowing the string to contain leading
/// or trailing whitespace and/or lowercase letters as long as it is otherwise the right length
/// and format.
pub fn parse_loose(value: &str) -> Result<LEI, LEIError> {
    let uc = value.to_ascii_uppercase();
    let temp = uc.trim();
    parse(temp)
}

/// Build an LEI from a _Payload_ (an already-concatenated _LOU ID_ and _Entity ID_). The
/// _Check Digits_ are automatically computed.
pub fn build_from_payload(payload: &str) -> Result<LEI, LEIError> {
    if payload.len() != 18 {
        return Err(LEIError::InvalidPayloadLength { was: payload.len() });
    }
    let b = &payload.as_bytes()[0..18];

    let lou_id = &b[0..4];
    validate_lou_id_format(lou_id)?;

    let entity_id = &b[4..18];
    validate_entity_id_format(entity_id)?;

    let mut bb = [0u8; 20];

    bb[0..18].copy_from_slice(b);
    let temp = compute_check_digits(b);
    bb[18..20].copy_from_slice(&temp);

    Ok(LEI(bb))
}

/// Build an LEI from its parts: an _LOU ID_ and an _Entity ID_. The _Check Digits_ are
/// automatically computed.
pub fn build_from_parts(lou_id: &str, entity_id: &str) -> Result<LEI, LEIError> {
    if lou_id.len() != 4 {
        return Err(LEIError::InvalidLouIdLength { was: lou_id.len() });
    }
    let lou_id: &[u8] = &lou_id.as_bytes()[0..4];
    validate_lou_id_format(lou_id)?;

    if entity_id.len() != 14 {
        return Err(LEIError::InvalidEntityIdLength {
            was: entity_id.len(),
        });
    }
    let entity_id: &[u8] = &entity_id.as_bytes()[0..14];
    validate_entity_id_format(entity_id)?;

    let mut bb = [0u8; 20];

    bb[0..4].copy_from_slice(lou_id);
    bb[4..18].copy_from_slice(entity_id);
    let temp = compute_check_digits(&bb[0..18]);
    bb[18..20].copy_from_slice(&temp);

    Ok(LEI(bb))
}

/// Test whether or not the passed string is in valid LEI format, without producing an LEI struct
/// value.
pub fn validate(value: &str) -> bool {
    if value.len() != 20 {
        return false;
    }

    // We make the preliminary assumption that the string is pure ASCII, so we work with the
    // underlying bytes. If there is Unicode in the string, the bytes will be outside the
    // allowed range and format validations will fail.

    let b = value.as_bytes();

    // We slice out the three fields and validate their formats.

    let lou_id: &[u8] = &b[0..4];
    if validate_lou_id_format(lou_id).is_err() {
        return false;
    }

    let entity_id: &[u8] = &b[4..18];
    if validate_entity_id_format(entity_id).is_err() {
        return false;
    }

    let check_digits = &b[18..20];
    if validate_check_digits_format(check_digits).is_err() {
        return false;
    }

    let payload = &b[0..18];

    let computed_check_digits = compute_check_digits(payload);

    if check_digits[0] != computed_check_digits[0] {
        return false;
    }

    if check_digits[1] != computed_check_digits[1] {
        return false;
    }

    true
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/// An LEI in confirmed valid format.
///
/// You cannot construct an LEI value manually. This does not compile:
///
/// ```compile_fail
/// use lei;
/// let cannot_construct = lei::LEI([0_u8; 20]);
/// ```
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
#[repr(transparent)]
#[allow(clippy::upper_case_acronyms)]
pub struct LEI([u8; 20]);

impl fmt::Display for LEI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let temp = unsafe { self.as_bytes().to_str_unchecked() }; // This is safe because we know it is ASCII
        write!(f, "{temp}")
    }
}

impl fmt::Debug for LEI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let temp = unsafe { self.as_bytes().to_str_unchecked() }; // This is safe because we know it is ASCII
        write!(f, "LEI({temp})")
    }
}

impl FromStr for LEI {
    type Err = LEIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_loose(s)
    }
}

impl LEI {
    /// Internal convenience function for treating the ASCII characters as a byte-array slice.
    fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Return just the _LOU ID_ portion of the LEI.
    pub fn lou_id(&self) -> &str {
        unsafe { self.0[0..4].to_str_unchecked() } // This is safe because we know it is ASCII
    }

    /// Return just the _Entity ID_ portion of the LEI.
    pub fn entity_id(&self) -> &str {
        unsafe { self.0[4..18].to_str_unchecked() } // This is safe because we know it is ASCII
    }

    /// Return the _Payload_ &mdash; everything except the _Check Digits_.
    pub fn payload(&self) -> &str {
        unsafe { self.0[0..18].to_str_unchecked() } // This is safe because we know it is ASCII
    }

    /// Return just the _Check Digit_ portion of the ISIN.
    pub fn check_digits(&self) -> &str {
        unsafe { self.0[18..20].to_str_unchecked() } // This is safe because we know it is ASCII
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This is from the ISIN_LEI_20210209.csv file from GLEIF.
    #[test]
    fn check_digits() {
        let payload = "635400B4JJBON4TCHF";
        let cd = compute_check_digits(payload.as_bytes());
        assert_eq!(cd[0], 48); // ASCII digit '0'
        assert_eq!(cd[1], 50); // ASCII digit '2'
    }

    /// These are from the ISIN_LEI_20210209.csv file from GLEIF.
    #[test]
    fn parse_bulk() {
        let cases = [
            "635400B4JJBON4TCHF02",
            "529900ODI3047E2LIV03",
            "5493002F3N6V3Z14SP04",
            "549300IYKILIU506KA05",
            "JJKC32MCHWDI71265Z06",
            "549300RIPPWJB5Z0FK07",
            "Z2VZBHUMB7PWWJ63I008",
            "FRQ78DFDYWMT3XY6UR09",
            "337KMNHEWWWR6B7Q7W10",
            "549300E9PC51EN656011",
            "5493003WHB7TFLYQFS12",
            "549300C04BJ0G297NC13",
            "T68X8LLAQYRNDV034K14",
            "8HWWA59ZS6Z54QLX6S15",
            "54930018SOOHBHRLWC16",
            "95980020140005346817",
            "549300HMMEWVG3PPQU18",
            "5JQ7W3GWO8J5DAE5WR19",
            "AJ6VL0Z1WDC42KKJZO20",
        ];

        for case in cases.iter() {
            let parsed = parse(case).unwrap();
            assert_eq!(
                *case,
                parsed.to_string(),
                "Successfully parsed {:?} but to_string() didn't match input!",
                case
            );
            let is_valid = validate(case);
            assert_eq!(
                true, is_valid,
                "Successfully parsed {:?} but got false from validate()!",
                case
            );
        }
    }

    /// These come from the ISIN_LEI_20210209.csv file from GLEIF. Note that according to the ISO
    /// standard itself, section 5 "Check digit pair", subsection 5.1 "General": "00, 01 and 99 are
    /// not valid LEI check digit pairs".
    #[test]
    fn parse_bulk_bad() {
        let cases = [
            "31570010000000045200",
            "3157006B6JVZ5DFMSN00",
            "315700BBRQHDWX6SHZ00",
            "315700G5G24XYL1TXH00",
            "31570010000000048401",
            "31570010000000067801",
            "315700WH3YMKHCVYW201",
        ];

        for case in cases.iter() {
            let parsed = parse(case).unwrap();
            assert_eq!(
                *case,
                parsed.to_string(),
                "Successfully parsed {:?} but to_string() didn't match input!",
                case
            );
            let is_valid = validate(case);
            assert_eq!(
                true, is_valid,
                "Successfully parsed {:?} but got false from validate()!",
                case
            );
        }
    }
}
