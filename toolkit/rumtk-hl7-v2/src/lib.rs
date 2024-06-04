pub mod hl7_v2_parser;
mod hl7_v2_interpreter;
mod hl7_v2_constants;
mod hl7_v2_types;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
