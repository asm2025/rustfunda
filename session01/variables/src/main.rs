use util::io::input;

fn main() {
    loop {
        let input = input(Some("Enter some text (or press Enter to quit): "));

        match input {
            Ok(value) => {
                if value.is_empty() {
                    println!("Exiting...");
                    break;
                }

                if value.len() > 50 {
                    println!("Warning: Input is quite long ({} characters)", value.len());
                }

                println!("You entered: {}", value);
            }
            Err(ex) => {
                println!("{}", ex);
                break;
            }
        }
    }
}
