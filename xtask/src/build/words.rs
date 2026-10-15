pub fn is_part_of_a_word(line: &str, start: usize, term: &str) -> bool {
    let is_word_character = |character: char| character.is_alphanumeric() || character == '_';
    let whole_word_only =
        term.starts_with(char::is_alphanumeric) && term.ends_with(char::is_alphanumeric);
    if !whole_word_only {
        return false;
    }
    let before = line[..start].chars().next_back();
    let after = line[start + term.len()..].chars().next();
    before.is_some_and(is_word_character) || after.is_some_and(is_word_character)
}
