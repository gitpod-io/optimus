pub fn vowel_gen(sentense: &String) -> String {
    let first_char = sentense.chars().next();
    let mut is_vowel = false;

    if first_char.is_some() {
        for vowel in ["a", "e", "i", "o", "u"].iter() {
            let first_char_low = first_char.unwrap().to_lowercase().to_string();
            let vowel_string = vowel.to_string();

            if first_char_low == vowel_string {
                is_vowel = true;
                break;
            }
        }
    }

    if is_vowel {
        "An".to_string()
    } else {
        "A".to_string()
    }
}
