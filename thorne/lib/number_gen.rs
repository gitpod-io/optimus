use rand::Rng;

pub fn _number(digits: i128, times: i128) -> String {
    let mut num: String = String::from("");

    for i in 0..times {
        for _ in 0..digits {
            num = num + &(rand::thread_rng().gen_range(0..9).to_string());
        }
        if i + 1 != times {
            num = String::from(num + "\n");
        }
    }

    return num;
}