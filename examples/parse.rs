fn main() {
    let lei_string = "YZ83GD8L7GG84979J516"; // Example from Section A.1 of The Standard
    match lei::parse(lei_string) {
        Ok(lei) => {
            println!("Parsed LEI: {}", lei); // "YZ83GD8L7GG84979J516"
            println!("  LOU ID: {}", lei.lou_id()); // "YZ83"
            println!("  Entity ID: {}", lei.entity_id()); // "GD8L7GG84979J5"
            println!("  Check digits: {}", lei.check_digits()); // "16"
        }
        Err(err) => panic!("Unable to parse LEI {}: {}", lei_string, err),
    }
}
