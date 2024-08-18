use core::panic;
use std::collections::HashSet;
use std::env;
use std::io;
use std::process;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if input_line.lines().any(|line| match_exists(line, &pattern)) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CharacterClass {
    Literal(char),
    CharGroup(String),
    NegativeCharGroup(String),
    Digit,
    Alphanumeric,
    StartAnchor(char),
    EndAnchor(Box<CharacterClass>),
}

fn parse_pattern(pattern: &str) -> Vec<CharacterClass> {
    let mut char_classes: Vec<CharacterClass> = Vec::new();

    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next().unwrap() {
                'd' => char_classes.push(CharacterClass::Digit),
                'w' => char_classes.push(CharacterClass::Alphanumeric),
                '\\' => char_classes.push(CharacterClass::Literal('\\')),
                '[' => char_classes.push(CharacterClass::Literal('[')),
                '^' => char_classes.push(CharacterClass::Literal('^')),
                '$' => char_classes.push(CharacterClass::Literal('$')),
                _ => panic!("invalid charactr class"),
            },
            '[' => match chars.peek().unwrap() {
                '^' => {
                    chars.next();

                    let char_group = collect_until(&mut chars, ']');
                    char_classes.push(CharacterClass::NegativeCharGroup(char_group));
                }
                _ => {
                    let char_group = collect_until(&mut chars, ']');
                    char_classes.push(CharacterClass::CharGroup(char_group));
                }
            },
            '^' => {
                let starting_char = chars.next().unwrap();
                if !char_classes.is_empty() {
                    panic!("start of string (^) anchor must be declared as the first pattern");
                }
                char_classes.push(CharacterClass::StartAnchor(starting_char));
            }
            '$' => {
                // TODO:
                // if !char_classes.is_empty() {
                //     panic!("start fo string (^) anchor must be declared as the first pattern");
                // }

                // check if $ anchor is last pattern
                if chars.peek().is_some() {
                    panic!("end of string ($) anchor must be declared as the last pattern");
                }
                if let Some(char_class) = char_classes.pop() {
                    char_classes.push(CharacterClass::EndAnchor(Box::new(char_class)));
                }
            }
            literal => char_classes.push(CharacterClass::Literal(literal)),
        }
    }

    char_classes
}

fn collect_until<I>(chars: &mut I, end_char: char) -> String
where
    I: Iterator<Item = char>,
{
    let mut result = String::new();
    for ch in chars.by_ref() {
        if ch == end_char {
            return result;
        }
        result.push(ch);
    }
    panic!("unclosed group, expected '{}'", end_char)
}

fn match_exists(input_line: &str, pattern: &str) -> bool {
    let patterns = parse_pattern(pattern);

    if let CharacterClass::StartAnchor(_) = patterns[0] {
        if !match_pattern(&input_line[0..1], &patterns[0]) {
            return false;
        }
    }
    if let CharacterClass::EndAnchor(_) = patterns[patterns.len() - 1] {
        if !match_pattern(
            &input_line[input_line.len() - 1..input_line.len()],
            &patterns[patterns.len() - 1],
        ) {
            return false;
        }
    }

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
                if let Some(CharacterClass::EndAnchor(_)) = patterns.last() {
                    // check if it's the end of the line
                    if source.chars().nth(j + 1).is_some() {
                        break;
                    }
                }
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
        CharacterClass::StartAnchor(char) => input_line.starts_with(&char.to_string()),
        CharacterClass::EndAnchor(char_class) => match_pattern(input_line, char_class),
    }
}

#[cfg(test)]
mod test {
    use crate::{match_exists, parse_pattern, CharacterClass};

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
    fn test_start_of_line_anchor_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = r"^J\w\w\w";

        assert!(match_exists(input, pattern));
    }

    #[test]
    fn test_start_of_line_anchor_mismatch() {
        let input = "slog";
        let pattern = r"^log";

        assert!(!match_exists(input, pattern));
    }

    #[test]
    fn test_end_of_string_anchor_literal_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = r"ry$";

        assert!(match_exists(input, pattern));
    }

    #[test]
    fn test_end_of_string_anchor_alphanumeric_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = r"\w$";

        assert!(match_exists(input, pattern));
    }
}
