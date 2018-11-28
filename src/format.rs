use failure::Error;
use std::fmt;

const MAX_LINE_LENGTH: usize = 80;
const INDENT_SIZE: usize = 2;

pub mod query;
pub mod schema;

pub struct Indentation {
    size: usize,
    count: usize,
}

impl Indentation {
    fn new(size: usize) -> Indentation {
        Indentation { size, count: 0 }
    }

    fn decrement(&mut self) {
        self.count -= 1;
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn spaces(&self) -> String {
        let mut indent = String::new();
        for _ in 0..self.count * self.size {
            indent.push_str(" ");
        }
        indent
    }
}

pub struct Output {
    buf: String,
}

impl Output {
    fn new() -> Output {
        Output { buf: String::new() }
    }

    fn push<T: AsRef<str>>(&mut self, s: T, indent: &Indentation) {
        self.push_str(format!("{}{}", indent.spaces(), s.as_ref()));
    }

    fn push_str<T: AsRef<str>>(&mut self, s: T) {
        self.buf.push_str(s.as_ref());
    }

    fn current_line(&self) -> &str {
        self.buf.lines().last().unwrap_or("")
    }

    fn current_line_length(&self) -> usize {
        self.current_line().len()
    }

    fn trim(&self) -> &str {
        self.buf.trim()
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.buf)
    }
}

pub fn map_join<I, T, K, F>(
    iter: I,
    mapper: F,
    sep: &str,
    out: &mut Output,
) where
    I: Iterator<Item = T>,
    F: Fn(T) -> K,
    T: std::fmt::Display,
    K: std::fmt::Display,
{
    let joined = iter
        .map(|thing| format!("{}", mapper(thing)))
        .collect::<Vec<_>>()
        .join(sep);
    out.push_str(&joined);
}

#[cfg(test)]
pub fn format_test<F>(formatter: F, query: &str, expected: &str)
where
    F: Fn(&str) -> Result<String, Error>,
{
    let query = query.trim();
    let actual = formatter(query).unwrap();

    let expected = expected.trim();

    if actual != expected {
        println!("--- Actual:\n{}", actual);
        println!("--- Expected:\n{}", expected);
        panic!("expected != actual");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indentation() {
        let mut indent = Indentation::new(2);
        indent.increment();
        indent.increment();
        indent.decrement();

        assert_eq!(indent.spaces(), "  ");
    }

    #[test]
    fn test_output() {
        let mut out = Output::new();
        let mut indent = Indentation::new(2);
        indent.increment();
        out.push("a", &indent);

        assert_eq!(out.to_string(), "  a");
    }

    #[test]
    fn test_output_current_line() {
        let mut out = Output::new();
        let mut indent = Indentation::new(2);

        assert_eq!(out.current_line(), "");
        out.push_str("hi");
        assert_eq!(out.current_line(), "hi");
        out.push_str("ho\n");
        assert_eq!(out.current_line(), "hiho");
        out.push_str("hux");
        assert_eq!(out.current_line(), "hux");
    }

    #[test]
    fn trim_output() {
        let mut out = Output::new();
        let mut indent = Indentation::new(0);
        out.push("\n   a  \t  \n", &indent);

        assert_eq!(out.trim(), "a");
    }
}
