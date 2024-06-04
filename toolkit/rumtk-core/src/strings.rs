pub fn count_tokens_ignoring_pattern(vector: &Vec<&str>, string_token: &String) -> usize {
    let mut count: usize = 0;
    for tok in vector.iter() {
        if string_token != tok {
            count += 1;
        }
    }
    count
}