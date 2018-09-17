// kept you waiting huh
pub fn snake_case(string: &str) -> String {
    let mut new_string = string.to_owned();
    let lowercase = &new_string[0..1].to_lowercase();
    new_string.replace_range(0..1, lowercase);
    let vec: Vec<_> = new_string
        .match_indices(char::is_uppercase)
        .map(|(i, character)| (i, character.to_owned()))
        .collect();
    for (i, (j, character)) in vec.into_iter().enumerate() {
        let mut lowercase = character.to_lowercase();
        lowercase.insert_str(0, "_");
        new_string.replace_range(i + j..=i + j, &lowercase);
    }
    new_string
}

pub fn capitalize(string: &mut String) {
    let uppercase = string[0..1].to_uppercase();
    string.replace_range(0..1, &uppercase)
}
