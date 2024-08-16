use std::collections::HashSet;
use std::env;
use std::io;
use std::process;

#[derive(Debug)]
enum CharacterClass {
    CharGroup(String),
    NegativeCharGroup(String),
    Digit,
    Alphanumeric,
}

impl CharacterClass {
    fn into_character_class(pattern: &str) -> Self {
        match pattern {
            p if p.starts_with("[^") && p.ends_with(']') => {
                CharacterClass::NegativeCharGroup(p[2..p.len() - 1].chars().collect())
            }
            p if p.starts_with('[') && p.ends_with(']') => {
                CharacterClass::CharGroup(p[1..p.len() - 1].chars().collect())
            }
            r"\d" => CharacterClass::Digit,
            r"\w" => CharacterClass::Alphanumeric,
            _ => panic!("Unhandled pattern: {}", pattern),
        }
    }

    // fn parse_pattern(pattern: &str) -> Self {}
}

fn match_exists(input_line: &str, pattern: &str) -> bool {
    let patterns = extract_pattern(pattern);

    for i in 0..input_line.len() {
        if !match_pattern(&input_line[i..i + 1], &patterns[0]) {
            continue;
        }

        let source = &input_line[i..];

        for (j, ch) in source.chars().enumerate() {
            if !match_pattern(&ch.to_string(), &patterns[j]) {
                break;
            }

            if j == patterns.len() - 1 {
                return true;
            }
        }
    }

    false
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        p if p.chars().count() == 1 => input_line.contains(p),
        p if p.starts_with("[^") && p.ends_with(']') => {
            let set: HashSet<char> = p[2..p.len() - 1].chars().collect();
            !set.iter().any(|&char| input_line.contains(char))
        }
        p if p.starts_with('[') && p.ends_with(']') => {
            let set: HashSet<char> = p[1..p.len() - 1].chars().collect();
            set.iter().any(|&char| input_line.contains(char))
        }
        r"\d" => input_line.contains(|x: char| x.is_ascii_digit()),
        r"\w" => input_line.contains(|x: char| x.is_alphanumeric()),
        _ => panic!("Unhandled pattern: {}", pattern),
    }
}

fn extract_pattern(pattern: &str) -> Vec<String> {
    let mut chars = pattern.chars();

    let mut pattern: Vec<String> = Vec::new();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let char_class = format!("{}{}", ch.to_string(), chars.next().unwrap());
            pattern.push(char_class);
        } else {
            pattern.push(ch.to_string());
        }
    }

    pattern
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_exists(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[cfg(test)]
mod test {
    use crate::{extract_pattern, match_exists};

    #[test]
    fn test_extract_pattern() {
        let pattern = "\\d\\d\\d apple";
        assert_eq!(
            extract_pattern(pattern),
            vec![
                "\\d".to_string(),
                "\\d".to_string(),
                "\\d".to_string(),
                " ".to_string(),
                "a".to_string(),
                "p".to_string(),
                "p".to_string(),
                "l".to_string(),
                "e".to_string()
            ]
        );
    }

    #[test]
    fn test_match_exists_end_to_end_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\d\\d\\d years";

        assert!(match_exists(input, pattern));
    }

    #[test]
    fn test_match_exists_alphanumeric_combination_partial_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\w\\d yea";

        assert!(match_exists(input, pattern));
    }

    #[test]
    fn test_match_exists_no_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\d\\d lorem";

        assert!(!match_exists(input, pattern));
    }

    #[test]
    fn test_match_exists_digit_alphanumeric_literal_combination() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\d \\w\\w\\w\\ws";

        assert!(match_exists(input, pattern));
    }
}
