pub fn remove_all_special(text: &str) -> String {
        // replace all specials with empty
    text.replace(&['(', ')', '<', '>', ',', '\"', '.', ';', ':', '\'', '\n', '\r'][..], "")
        .replace(" ", "_") //replace whitespace with underscore for readability

}
