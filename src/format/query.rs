use super::{Indentation, Output};
use failure::{bail, Error};
use graphql_parser::{parse_query, query::*};

const MAX_LINE_LENGTH: usize = 80;
const INDENT_SIZE: usize = 2;

pub fn format(contents: &str) -> Result<String, Error> {
    let ast = parse_query(contents)?;

    let mut out = Output::new();
    let mut indent = Indentation::new(INDENT_SIZE);
    format_doc(ast, &mut indent, &mut out);

    Ok(out.trim().to_string())
}

fn format_doc(doc: Document, indent: &mut Indentation, out: &mut Output) {
    for def in doc.definitions {
        format_def(def, indent, out);
    }
}

fn format_def(def: Definition, indent: &mut Indentation, out: &mut Output) {
    match def {
        Definition::Operation(operation) => format_operation(operation, indent, out),
        Definition::Fragment(fragment) => todo!("Fragment"),
    }
}

fn format_operation(op: OperationDefinition, indent: &mut Indentation, out: &mut Output) {
    match op {
        OperationDefinition::SelectionSet(set) => todo!("selection set"),

        OperationDefinition::Query(query) => {
            todo_field!(query, variable_definitions);
            todo_field!(query, directives);

            if let Some(name) = query.name {
                out.push(&format!("query {name}", name = name), indent);
            } else {
                out.push("query", indent);
            }
            format_selection_set(query.selection_set, indent, out);
            out.push_str("\n");
        }

        OperationDefinition::Mutation(mutation) => todo!("mutation"),

        OperationDefinition::Subscription(sub) => todo!("sub"),
    }
}

fn format_selection_set(set: SelectionSet, indent: &mut Indentation, out: &mut Output) {
    let items = set.items;

    if items.is_empty() {
        return;
    }

    out.push_str(" {\n");
    indent.increment();
    for selection in items {
        match selection {
            Selection::Field(field) => format_field(field, indent, out),
            Selection::FragmentSpread(frag_spread) => todo!("frag_spread"),
            Selection::InlineFragment(inline_frag) => todo!("inline_frag"),
        }
    }
    indent.decrement();
    out.push("}\n", indent);
}

fn format_field(field: Field, indent: &mut Indentation, out: &mut Output) {
    todo_field!(field, directives);

    if let Some(alias) = field.alias {
        out.push(
            &format!("{alias}: {name}", alias = alias, name = field.name),
            indent,
        );
    } else {
        out.push(&format!("{name}", name = field.name), indent);
    }

    if !field.arguments.is_empty() {
        out.push_str("(");
        let current_line_length = out.current_line_length();

        let args = field
            .arguments
            .iter()
            .map(|(key, value)| format!("{arg}: {value}", arg = key, value = value.to_string()))
            .collect::<Vec<_>>();
        let args_joined = args.join(", ") + ")";

        let line_length_with_args = current_line_length + args_joined.len();

        if line_length_with_args > MAX_LINE_LENGTH {
            indent.increment();
            out.push_str("\n");
            args.iter().for_each(|arg| {
                out.push(&format!("{},\n", arg), indent);
            });
            indent.decrement();
            out.push(")", indent);
        } else {
            out.push_str(&args_joined);
        }
    }

    if field.selection_set.items.is_empty() {
        out.push_str("\n");
    } else {
        format_selection_set(field.selection_set, indent, out);
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_basic() {
        let query = "
query One { firstName }
query Two { firstName lastName }
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query One {
  firstName
}

query Two {
  firstName
  lastName
}
            "
        .trim();

        if actual != expected {
            println!("Actual:\n{}", actual);
            println!("Expected:\n{}", expected);
            panic!("expected != actual");
        }
    }

    #[test]
    fn test_nested_queries() {
        let query = "
query One { firstName }
query Two { firstName lastName team { id slug league { id } } country { id } }
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query One {
  firstName
}

query Two {
  firstName
  lastName
  team {
    id
    slug
    league {
      id
    }
  }
  country {
    id
  }
}
            "
        .trim();

        if actual != expected {
            println!("Actual:\n\n{}\n", actual);
            println!("Expected:\n\n{}", expected);
            panic!("expected != actual");
        }
    }

    #[test]
    fn alias() {
        let query = "
query One { alias: firstName }
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query One {
  alias: firstName
}
            "
        .trim();

        if actual != expected {
            println!("Actual:\n\n{}\n", actual);
            println!("Expected:\n\n{}", expected);
            panic!("expected != actual");
        }
    }

    #[test]
    fn args() {
        let query = "
query One { firstName(a: { b: 123, one:ONE }) { id } }
query Two { firstName(a: \"123\") { id } }
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query One {
  firstName(a: {b: 123, one: ONE}) {
    id
  }
}

query Two {
  firstName(a: \"123\") {
    id
  }
}
            "
        .trim();

        if actual != expected {
            println!("Actual:\n\n{}\n", actual);
            println!("Expected:\n\n{}", expected);
            panic!("expected != actual");
        }
    }

    #[test]
    fn args_long_lines() {
        let query = "
query UserProfile {
  user(a: 123, a: 123, a: 123, a: 123, a: 123, a: 123, a: 123, a: 123, a: 123, a: 123) {
    team
  }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query UserProfile {
  user(
    a: 123,
    a: 123,
    a: 123,
    a: 123,
    a: 123,
    a: 123,
    a: 123,
    a: 123,
    a: 123,
    a: 123,
  ) {
    team
  }
}
            "
        .trim();

        if actual != expected {
            println!("Actual:\n\n{}\n", actual);
            println!("Expected:\n\n{}", expected);
            panic!("expected != actual");
        }
    }
}
