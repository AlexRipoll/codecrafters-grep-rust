use core::panic;
use std::collections::HashSet;
use std::env;
use std::io;
use std::iter::Peekable;
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

    if input_line
        .lines()
        .any(|line| pattern_matches(line, &pattern))
    {
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
    OneOrMore(Box<CharacterClass>),
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
                '+' => char_classes.push(CharacterClass::Literal('+')),
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
                // check if $ anchor is last pattern
                if chars.peek().is_some() {
                    panic!("end of string ($) anchor must be declared as the last pattern");
                }

                if let Some(char_class) = char_classes.pop() {
                    char_classes.push(CharacterClass::EndAnchor(Box::new(char_class)));
                }
            }
            '+' => {
                if char_classes.is_empty() {
                    panic!("quantifier (+) must be declared after another character class");
                }

                if let Some(char_class) = char_classes.pop() {
                    char_classes.push(CharacterClass::OneOrMore(Box::new(char_class)));
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

fn pattern_matches(input_str: &str, pattern_str: &str) -> bool {
    let patterns = parse_pattern(pattern_str);

    if let Some(CharacterClass::StartAnchor(_)) = patterns.get(0) {
        if !match_pattern(&input_str[0..1], &patterns[0]) {
            return false;
        }
    }

    if let Some(CharacterClass::EndAnchor(_)) = patterns.last() {
        if !match_pattern(
            &input_str[input_str.len() - 1..input_str.len()],
            &patterns.last().unwrap(),
        ) {
            return false;
        }
    }

    for i in 0..input_str.len() {
        let mut pattern_idx = 0;
        let mut input_chars = input_str[i..].chars().peekable();

        while let Some(ch) = input_chars.next() {
            if !match_pattern(&ch.to_string(), &patterns[pattern_idx]) {
                break;
            }

            handle_quantifier(&mut input_chars, &patterns[pattern_idx]);

            if pattern_idx == patterns.len() - 1 {
                if let Some(CharacterClass::EndAnchor(_)) = patterns.last() {
                    if input_chars.peek().is_some() {
                        break;
                    }
                }
                return true;
            }

            pattern_idx += 1;
        }
    }

    false
}

fn handle_quantifier<I>(char_iter: &mut Peekable<I>, pattern: &CharacterClass)
where
    I: Iterator<Item = char>,
{
    match pattern {
        CharacterClass::OneOrMore(char_class) => {
            while let Some(next_ch) = char_iter.peek() {
                if match_pattern(&next_ch.to_string(), char_class) {
                    char_iter.next();
                } else {
                    break;
                }
            }
        }
        _ => {}
    }
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
        CharacterClass::OneOrMore(char_class) => match_pattern(input_line, char_class),
    }
}

#[cfg(test)]
mod test {
    use crate::{parse_pattern, pattern_matches, CharacterClass};

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

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_match_exists_alphanumeric_combination_partial_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\w\\d yea";

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_match_exists_no_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\d\\d lorem";

        assert!(!pattern_matches(input, pattern));
    }

    #[test]
    fn test_match_exists_digit_alphanumeric_literal_combination() {
        let input = "John Doe has more than 700 years of history";
        let pattern = "\\d \\w\\w\\w\\ws";

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_excaped_back_slash() {
        let input = "sally has 12 apples";
        let pattern = r"\d\\d\\d apples";

        assert!(!pattern_matches(input, pattern));
    }

    #[test]
    fn test_start_of_line_anchor_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = r"^J\w\w\w";

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_start_of_line_anchor_mismatch() {
        let input = "slog";
        let pattern = r"^log";

        assert!(!pattern_matches(input, pattern));
    }

    #[test]
    fn test_end_of_string_anchor_literal_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = r"ry$";

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_end_of_string_anchor_alphanumeric_match() {
        let input = "John Doe has more than 700 years of history";
        let pattern = r"\w$";

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_end_of_string_anchor_digit_match() {
        let input = "John Doe has more than 700 years of history :3";
        let pattern = r"\d$";

        assert!(pattern_matches(input, pattern));
    }

    #[test]
    fn test_end_of_string_anchor_mismatch() {
        let input = "cat, dog and more dogs";
        let pattern = r"dog$";

        assert!(!pattern_matches(input, pattern));
    }

    #[test]
    fn test_one_or_more_literal_match() {
        let input = "Hellooooooooooo wo!";
        let pattern = r"\w\wo+ \w";

        assert!(pattern_matches(input, pattern));
    }
}
