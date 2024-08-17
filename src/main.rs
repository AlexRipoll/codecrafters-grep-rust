use core::panic;
use std::collections::HashSet;
use std::env;
use std::io;
use std::process;

#[derive(Debug, PartialEq, Eq)]
enum CharacterClass {
    Literal(char),
    CharGroup(String),
    NegativeCharGroup(String),
    Digit,
    Alphanumeric,
}

fn parse_pattern(pattern: &str) -> Vec<CharacterClass> {
    let mut char_classes: Vec<CharacterClass> = Vec::new();

    let mut chars = pattern.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next().unwrap() {
                'd' => char_classes.push(CharacterClass::Digit),
                'w' => char_classes.push(CharacterClass::Alphanumeric),
                '\\' => char_classes.push(CharacterClass::Literal('\\')),
                _ => panic!("invalid charactr class"),
            },
            '[' => match chars.next().unwrap() {
                '^' => {
                    let mut char_group: String = String::from("");

                    while let Some(ch) = chars.next() {
                        match ch {
                            ']' => {
                                char_classes.push(CharacterClass::NegativeCharGroup(char_group));
                                break;
                            }
                            literal => {
                                char_group = format!("{}{}", char_group, literal);
                            }
                        }
                    }
                }
                n => {
                    let mut char_group: String = String::from(n);

                    while let Some(ch) = chars.next() {
                        match ch {
                            ']' => {
                                char_classes.push(CharacterClass::CharGroup(char_group));
                                break;
                            }
                            literal => {
                                char_group = format!("{}{}", char_group, literal);
                            }
                        }
                    }
                }
            },

            literal => char_classes.push(CharacterClass::Literal(literal)),
        }
    }

    char_classes
}

fn match_exists(input_line: &str, pattern: &str) -> bool {
    // let patterns = extract_pattern(pattern);
    let patterns = parse_pattern(pattern);

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

fn match_pattern(input_line: &str, pattern: &CharacterClass) -> bool {
    match pattern {
        CharacterClass::Literal(char) => input_line.contains(*char),
        CharacterClass::NegativeCharGroup(group) => {
            let set: HashSet<char> = group.chars().collect();
            !set.iter().any(|&char| input_line.contains(char))
        }
        CharacterClass::CharGroup(group) => {
            let set: HashSet<char> = group.chars().collect();
            set.iter().any(|&char| input_line.contains(char))
        }
        CharacterClass::Digit => input_line.contains(|x: char| x.is_ascii_digit()),
        CharacterClass::Alphanumeric => input_line.contains(|x: char| x.is_alphanumeric()),
    }
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
    use crate::{match_exists, parse_pattern, CharacterClass};

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

    #[test]
    fn test_excaped_back_slash() {
        let input = "sally has 12 apples";
        let pattern = r"\d\\d\\d apples";

        assert!(!match_exists(input, pattern));
    }

    #[test]
    fn test_parse_regexp() {
        let pattern = r"\d\d\w [abc] 0xab[^xyz]";
        assert_eq!(
            parse_pattern(pattern),
            vec![
                CharacterClass::Digit,
                CharacterClass::Digit,
                CharacterClass::Alphanumeric,
                CharacterClass::Literal(' '),
                CharacterClass::CharGroup("abc".to_string()),
                CharacterClass::Literal(' '),
                CharacterClass::Literal('0'),
                CharacterClass::Literal('x'),
                CharacterClass::Literal('a'),
                CharacterClass::Literal('b'),
                CharacterClass::NegativeCharGroup("xyz".to_string()),
            ]
        );
    }
}
