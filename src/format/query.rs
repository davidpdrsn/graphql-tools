use super::Indentation;
use failure::{bail, Error};
use graphql_parser::{parse_query, query::*};

pub fn format(contents: &str) -> Result<String, Error> {
    let ast = parse_query(contents)?;

    let mut out = String::new();
    let mut indent = Indentation::new(2);
    format_doc(ast, &mut indent, &mut out);

    Ok(out.trim().to_string())
}

fn format_doc(doc: Document, indent: &mut Indentation, out: &mut String) {
    for def in doc.definitions {
        format_def(def, indent, out);
    }
}

fn format_def(def: Definition, indent: &mut Indentation, out: &mut String) {
    match def {
        Definition::Operation(operation) => format_operation(operation, indent, out),
        Definition::Fragment(fragment) => todo!("Fragment"),
    }
}

fn format_operation(op: OperationDefinition, indent: &mut Indentation, out: &mut String) {
    match op {
        OperationDefinition::SelectionSet(set) => todo!("selection set"),

        OperationDefinition::Query(query) => {
            todo_field!(query, variable_definitions);
            todo_field!(query, directives);

            if let Some(name) = query.name {
                push(&format!("query {name}", name = name), indent, out);
            } else {
                push("query", indent, out);
            }
            format_selection_set(query.selection_set, indent, out);
            out.push_str("\n");
        }

        OperationDefinition::Mutation(mutation) => todo!("mutation"),

        OperationDefinition::Subscription(sub) => todo!("sub"),
    }
}

fn format_selection_set(set: SelectionSet, indent: &mut Indentation, out: &mut String) {
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
    push("}\n", indent, out);
}

fn format_field(field: Field, indent: &mut Indentation, out: &mut String) {
    todo_field!(field, directives);

    if let Some(alias) = field.alias {
        push(
            &format!("{alias}: {name}", alias = alias, name = field.name),
            indent,
            out,
        );
    } else {
        push(&format!("{name}", name = field.name), indent, out);
    }

    if !field.arguments.is_empty() {
        out.push_str("(");
        let args = field
            .arguments
            .iter()
            .map(|(key, value)| {
                format!("{arg}: {value}", arg = key, value = value.to_string())
            })
            .collect::<Vec<_>>();
        out.push_str(&args.join(", "));
        out.push_str(")");
    }

    if field.selection_set.items.is_empty() {
        out.push_str("\n");
    } else {
        format_selection_set(field.selection_set, indent, out);
    }
}

fn push(s: &str, indent: &Indentation, out: &mut String) {
    out.push_str(&format!("{spaces}{s}", spaces = indent.spaces(), s = s));
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
}
