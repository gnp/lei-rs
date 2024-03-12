lei
====
An `LEI` type for working with validated Legal Entity Identifiers (LEIs) as defined in
[ISO 17442:2020](https://www.iso.org/standard/78829.html) "Financial services — Legal entity identifier (LEI) — Part 1:
Assignment".

This crate is part of the Financial Identifiers series:

* [CIK](https://crates.io/crates/cik): Central Index Key (SEC EDGAR)
* [CUSIP](https://crates.io/crates/cusip): Committee on Uniform Security Identification Procedures (ANSI X9.6-2020)
* [ISIN](https://crates.io/crates/isin): International Securities Identification Number (ISO 6166:2021)
* [LEI](https://crates.io/crates/lei): Legal Entity Identifier (ISO 17442:2020)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
lei = "0.2"
```


## Example

```rust
use lei;
let lei_string = "YZ83GD8L7GG84979J516"; // Example from Section A.1 of The Standard
match lei::parse(lei_string) {
    Ok(lei) => {
        println!("Parsed LEI: {}", lei.to_string()); // "YZ83GD8L7GG84979J516"
        println!("  LOU ID: {}", lei.lou_id()); // "YZ83"
        println!("  Entity ID: {}", lei.entity_id()); // "GD8L7GG84979J5"
        println!("  Check digits: {}", lei.check_digits()); // "16"
    }
    Err(err) => panic!("Unable to parse LEI {}: {}", lei_string, err),
}
```


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
