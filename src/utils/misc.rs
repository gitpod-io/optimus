pub fn vowel_gen(sentense: &str) -> &str {
    let first_char = sentense.chars().next();
    let mut is_vowel = false;

    if let Some(first_char) = first_char {
        for vowel in ["a", "e", "i", "o", "u"].iter() {
            let first_char_low = first_char.to_lowercase().to_string();
            let vowel_string = vowel.to_string();

            if first_char_low == vowel_string {
                is_vowel = true;
                break;
            }
        }
    }

    if is_vowel {
        "An"
    } else {
        "A"
    }
}
