use fancy_regex::Regex;

pub fn replace_addresses(line: String) -> String {
    let boguscoin_regex = Regex::new(r"(?<=^|\s)(7[a-zA-Z0-9]{25,34})(?=$|\s)").unwrap();
    let target_address = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

    boguscoin_regex
        .replace_all(&line, target_address)
        .to_string()
}
