// import random and file and io
use rand::Rng;
use std::*;
mod english;
mod number_gen;

// gen function
/*
    * generates random string
    * uses normal list
*/
pub fn gen(charnr: u32, times: u32) -> String {
    // char list
    let charlist: [&str; 94] = [ 
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y",
        "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
        "Y", "Z", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "_", "+", "=", ")", "(", "*", "&", "^", "%", "$", "#", "@",
        "!", "}", "[", "]", "}", "\"", "/", "?", "|", "\"", "\"", ":", ";", ".", ">", "<", ",", "`", "~"
    ];

    let mut out: String = String::from("");

    for i in  0..times {
        for _ in  0..charnr {
            let randomnr = rand::thread_rng().gen_range(0..charlist.len());
            out = String::from(out + charlist[randomnr]);
        }
        if i + 1 != times {
            out = String::from(out + "\n");
        }
    }
    return out;
}

// fullgen function
/*
    * generates random string
    * uses full list
*/
pub fn fullgen(charnr: u32, times: u32) -> String {
    // char list
    let charlist: [&str; 140] = [ 
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y",
        "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
        "Y", "Z", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "_", "+", "=", ")", "(", "*", "&", "^", "%", "$", "#", "@",
        "!", "}", "[", "]", "}", "\"", "/", "?", "|", "\"", "\"", ":", ";", ".", ">", "<", ",", "`", "~", "¡", "™", "£", "¢",
        "∞", "¶", "•", "ª", "º", "–", "≠", "‘", "æ", "«", "…", "π", "ø", "ˆ", "¨", "¥", "†", "®", "´", "∑", "œ", "“", "…", "¬", "˚",
        "∆", "˙", "©", "ƒ", "ß", "∂", "å", "÷", "≥", "≤", "µ", "˜", "∫", "√", "ç", "≈", "`"
    ];

    let mut out: String = String::from("");

    for i in  0..times {
        for _ in  0..charnr {
            let randomnr = rand::thread_rng().gen_range(0..charlist.len());
            out = String::from(out + charlist[randomnr]);
        }
        if i + 1 != times {
            out = String::from(out + "\n");
        }
    }
    return out;
}

// tofile function
/*
    * generates random string
    * prints to file
    * uses normal list
*/
pub fn tofile(charnr: u32, times: u32, filename: String) {
    // char list
    let charlist: [&str; 94] = [ 
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y",
        "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
        "Y", "Z", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "_", "+", "=", ")", "(", "*", "&", "^", "%", "$", "#", "@",
        "!", "}", "[", "]", "}", "\"", "/", "?", "|", "\"", "\"", ":", ";", ".", ">", "<", ",", "`", "~"
    ];

    let mut out: String = String::from("");

    for i in  0..times {
        for _ in  0..charnr {
            let randomnr = rand::thread_rng().gen_range(0..charlist.len());
            out = String::from(out + charlist[randomnr]);
        }
        if i + 1 != times {
            out = String::from(out + "\n");
        }
    }

    fs::write(filename, out)
        .expect("Unable to write file");
}

// tofile function
/*
    * generates random string
    * prints to file
    * uses full list
*/
pub fn tofile_full(charnr: u32, times: u32, filename: String) {
    // char list
    let charlist: [&str; 140] = [ 
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y",
        "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
        "Y", "Z", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "_", "+", "=", ")", "(", "*", "&", "^", "%", "$", "#", "@",
        "!", "}", "[", "]", "}", "\"", "/", "?", "|", "\"", "\"", ":", ";", ".", ">", "<", ",", "`", "~", "¡", "™", "£", "¢",
        "∞", "¶", "•", "ª", "º", "–", "≠", "‘", "æ", "«", "…", "π", "ø", "ˆ", "¨", "¥", "†", "®", "´", "∑", "œ", "“", "…", "¬", "˚",
        "∆", "˙", "©", "ƒ", "ß", "∂", "å", "÷", "≥", "≤", "µ", "˜", "∫", "√", "ç", "≈", "`"
    ];

    let mut out: String = String::from("");

    for i in  0..times {
        for _ in  0..charnr {
            let randomnr = rand::thread_rng().gen_range(0..charlist.len());
            out = String::from(out + charlist[randomnr]);
        }
        if i + 1 != times {
            out = String::from(out + "\n");
        }
    }

    fs::write(filename, out)
        .expect("Unable to write file");
}

pub fn english_gen(str_nr: u64, times: u64) -> String {
    let mut english_out = String::new();
    for _i in 0..times {
        for _j in 0..str_nr {
            english_out = english_out + &(english::eng()) + " ";
        }
        if _i + 1 != times {
            english_out = english_out + "\n";
        }
    }
    return english_out;
}

pub fn your_list(str_list: &mut [&str], charnr: u32, times: u32) -> String {
    let mut out = String::new();

    for _i in 0..times {
        for _j in 0..charnr {
            let randomnr = rand::thread_rng().gen_range(0..str_list.len());
            out = out + &(str_list[randomnr]) + " ";
        }
    }

    return out;
}

pub fn english_to_file(str_nr: u32, times: u32, filename: String) {
    let mut english_out = String::new();

    for _i in 0..times {
        for _j in 0..str_nr {
            english_out = english_out + &(english::eng());
        }
        if _i + 1 != times {
            english_out = english_out + "\n";
        }
    }

    english::eng_to_file(english_out, filename);
}

pub fn number(digits: i128, times: i128) -> String {
    let num = number_gen::_number(digits, times);

    return num
}