// Adapted from rustfmt https://github.com/rust-lang/rustfmt

use std::collections::VecDeque;
use std::io;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub enum DiffLine {
    Context(String),
    Expected(String),
    Resulting(String),
}

#[derive(Debug, PartialEq)]
pub struct Mismatch {
    pub line_number: u32,
    pub line_number_orig: u32,
    pub lines: Vec<DiffLine>,
}

impl Mismatch {
    fn new(line_number: u32, line_number_orig: u32) -> Mismatch {
        Mismatch {
            line_number,
            line_number_orig,
            lines: Vec::new(),
        }
    }
}

pub struct OutputWriter {
    terminal: Option<Box<term::Terminal<Output = io::Stdout>>>,
}

impl OutputWriter {
    pub fn new() -> Self {
        if let Some(t) = term::stdout() {
            return OutputWriter { terminal: Some(t) };
        } else {
            return OutputWriter { terminal: None };
        }
    }

    pub fn writeln(&mut self, msg: &str, color: Option<term::color::Color>) {
        match &mut self.terminal {
            Some(ref mut t) => {
                if let Some(color) = color {
                    t.fg(color).unwrap();
                }
                writeln!(t, "{}", msg).unwrap();
                if color.is_some() {
                    t.reset().unwrap();
                }
            }
            None => println!("{}", msg),
        }
    }
}

pub fn make_diff(expected: &str, actual: &str, context_size: usize) -> Vec<Mismatch> {
    let mut line_number = 1;
    let mut line_number_orig = 1;
    let mut context_queue: VecDeque<&str> = VecDeque::with_capacity(context_size);
    let mut lines_since_mismatch = context_size + 1;
    let mut results = Vec::new();
    let mut mismatch = Mismatch::new(0, 0);

    for result in diff::lines(expected, actual) {
        match result {
            diff::Result::Left(str) => {
                if lines_since_mismatch >= context_size && lines_since_mismatch > 0 {
                    results.push(mismatch);
                    mismatch = Mismatch::new(
                        line_number - context_queue.len() as u32,
                        line_number_orig - context_queue.len() as u32,
                    );
                }

                while let Some(line) = context_queue.pop_front() {
                    mismatch.lines.push(DiffLine::Context(line.to_owned()));
                }

                mismatch.lines.push(DiffLine::Resulting(str.to_owned()));
                line_number_orig += 1;
                lines_since_mismatch = 0;
            }
            diff::Result::Right(str) => {
                if lines_since_mismatch >= context_size && lines_since_mismatch > 0 {
                    results.push(mismatch);
                    mismatch = Mismatch::new(
                        line_number - context_queue.len() as u32,
                        line_number_orig - context_queue.len() as u32,
                    );
                }

                while let Some(line) = context_queue.pop_front() {
                    mismatch.lines.push(DiffLine::Context(line.to_owned()));
                }

                mismatch.lines.push(DiffLine::Expected(str.to_owned()));
                line_number += 1;
                lines_since_mismatch = 0;
            }
            diff::Result::Both(str, _) => {
                if context_queue.len() >= context_size {
                    let _ = context_queue.pop_front();
                }

                if lines_since_mismatch < context_size {
                    mismatch.lines.push(DiffLine::Context(str.to_owned()));
                } else if context_size > 0 {
                    context_queue.push_back(str);
                }

                line_number += 1;
                line_number_orig += 1;
                lines_since_mismatch += 1;
            }
        }
    }

    results.push(mismatch);
    results.remove(0);

    results
}

pub fn print_diff(diff: Vec<Mismatch>) {
    let line_terminator = "";

    let mut writer = OutputWriter::new();

    for mismatch in diff {
        for line in mismatch.lines {
            match line {
                DiffLine::Context(ref str) => {
                    writer.writeln(&format!(" {}{}", str, line_terminator), None)
                }
                DiffLine::Expected(ref str) => writer.writeln(
                    &format!("+{}{}", str, line_terminator),
                    Some(term::color::GREEN),
                ),
                DiffLine::Resulting(ref str) => writer.writeln(
                    &format!("-{}{}", str, line_terminator),
                    Some(term::color::RED),
                ),
            }
        }
    }
}
