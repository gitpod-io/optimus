use rand::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn eng() -> String {
    let filename = format!(
        "{}/{}",
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy(),
        "english.txt"
    );

    if !std::path::Path::new(&filename).exists() {
        std::fs::write(&filename, include_bytes!("english.txt")).unwrap();
    }

    // Open the file in read-only mode (ignoring errors).
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    // setting let english
    #[warn(unused_mut)]
    let mut english_ = vec![String::from(""); 370103];

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors.

        english_[_index] = line;
    }

    let rand_ = rand::thread_rng().gen_range(0..english_.len());
    let english: String = String::from(&english_[rand_]);

    return english;
}

pub fn eng_to_file(instr: String, filename: String) {
    std::fs::write(filename, instr).expect("Unable to write file");
}
