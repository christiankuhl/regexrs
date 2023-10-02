use core::convert::AsRef;

mod fsm;
mod parser;

use fsm::{State, Transitions, FSM};
use parser::parse_regex;

struct Regex {
    fsm: FSM,
}

impl Regex {
    pub fn compile<S: AsRef<str>>(regex: S) -> Self {
        let ast = parse_regex(regex.as_ref());
        println!("{:?}", ast);
        let mut fsm = FSM::new();
        for c in regex.as_ref().chars() {
            let mut ts = Transitions::default();
            ts[char_to_idx(c)] = State::Intermediate(fsm.final_state() + 1);
            fsm.push(ts);
        }
        Self { fsm }
    }
    pub fn matches<S: AsRef<str>>(&self, string: S) -> bool {
        println!("Matching '{}'...", string.as_ref());
        println!("Tgt state: {}", self.fsm.final_state());
        let mut state = State::Intermediate(0);
        for c in string.as_ref().chars() {
            print!("{state} -> ");
            state = self.fsm.next(state, c);
            print!("{state}\n");
            if state == State::Failed {
                return false;
            } else if state == State::Success {
                return true;
            }
        }
        print!("EOL: {state} -> ");
        state = self.fsm.next(state, '\n');
        print!("{state}\n");
        state == State::Success
    }
}

fn char_to_idx(c: char) -> usize {
    if c == '$' {
        return '\n' as usize;
    }
    c as usize
}

fn main() {
    const TEST_CASES: [(&str, &str, bool); 30] = [
        (r"a", "a", true),
        (r"cat", "Cat", true),
        (r"[aeiou]", "apple", true),
        (r"[^0-9]", "Hello World!", true),
        (r"ab*c", "ac", true),
        (r"ab+c", "abbc", true),
        (r"(ab)+c", "ababc", true),
        (r"apple|banana", "banana", true),
        (r"^Hello$", "Hello", true),
        (r"\d{3,5}", "12345", true),
        (r"\bword\b", "This is a word.", true),
        (r"a(?=b)", "abc", true),
        (r"a(?!b)", "axc", true),
        (r"[A-Za-z]", "Hello World", true),
        (r"a.*?b", "aabb", true),
        (r"a", "b", false),
        (r"cat", "dog", false),
        (r"[aeiou]", "xyz", false),
        (r"[^0-9]", "12345", false),
        (r"ab*c", "adc", false),
        (r"ab+c", "ac", false),
        (r"(ab)+c", "abcabc", false),
        (r"apple|banana", "cherry", false),
        (r"^Hello$", "Hello, World!", false),
        (r"\d{3,5}", "12", false),
        (r"\bword\b", "wording", false),
        (r"a(?=b)", "axb", false),
        (r"a(?!b)", "abc", false),
        (r"[A-Za-z]", "123", false),
        (r"a.*?b", "acb", false),
    ];

    for (t, _, _) in TEST_CASES.iter() {
        println!("{t}");
        let regex = Regex::compile(t);
    }
    // println!("{}", regex.fsm);
    // let test_cases = vec!["Hello, World!", "abc", "abcd", "xyz"];
    // for test in test_cases {
    //     println!("{test} => {result}", result = regex.matches(test));
    // }
}

#[cfg(test)]
mod tests {
    const TEST_CASES: [(&str, &str, bool); 30] = [
        (r"a", "a", true),
        (r"cat", "Cat", true),
        (r"[aeiou]", "apple", true),
        (r"[^0-9]", "Hello World!", true),
        (r"ab*c", "ac", true),
        (r"ab+c", "abbc", true),
        (r"(ab)+c", "ababc", true),
        (r"apple|banana", "banana", true),
        (r"^Hello$", "Hello", true),
        (r"\d{3,5}", "12345", true),
        (r"\bword\b", "This is a word.", true),
        (r"a(?=b)", "abc", true),
        (r"a(?!b)", "axc", true),
        (r"[A-Za-z]", "Hello World", true),
        (r"a.*?b", "aabb", true),
        (r"a", "b", false),
        (r"cat", "dog", false),
        (r"[aeiou]", "xyz", false),
        (r"[^0-9]", "12345", false),
        (r"ab*c", "adc", false),
        (r"ab+c", "ac", false),
        (r"(ab)+c", "abcabc", false),
        (r"apple|banana", "cherry", false),
        (r"^Hello$", "Hello, World!", false),
        (r"\d{3,5}", "12", false),
        (r"\bword\b", "wording", false),
        (r"a(?=b)", "axb", false),
        (r"a(?!b)", "abc", false),
        (r"[A-Za-z]", "123", false),
        (r"a.*?b", "acb", false),
    ];

    use super::*;
}
