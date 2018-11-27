pub mod query;
pub mod schema;

struct Indentation {
    size: u32,
    count: u32,
}

impl Indentation {
    fn new(size: u32) -> Indentation {
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
}
