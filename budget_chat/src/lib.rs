pub mod models;

use models::Member;

use std::collections::HashSet;

pub fn format_names(members: HashSet<Member>) -> String {
    let names = members
        .into_iter()
        .map(|x| x.name)
        .collect::<Vec<String>>()
        .join(", ");

    format!("* The room contains: {}", names)
}

pub fn is_valid_name(name: &str) -> bool {
    name.trim().chars().all(char::is_alphanumeric) && !name.trim().is_empty()
}
