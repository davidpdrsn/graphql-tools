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
        OperationDefinition::Query(query) => {
            format_operation_type(OperationType::Query(query), indent, out)
        }
        OperationDefinition::Mutation(query) => {
            format_operation_type(OperationType::Mutation(query), indent, out)
        }
        OperationDefinition::SelectionSet(set) => format_selection_set(set, indent, out),
        OperationDefinition::Subscription(sub) => {
            format_operation_type(OperationType::Subscription(sub), indent, out)
        }
    }
}

fn format_operation_type(r#type: OperationType, indent: &mut Indentation, out: &mut Output) {
    todo_field!(r#type.variable_definitions());
    todo_field!(r#type.directives());

    if let Some(name) = r#type.name() {
        out.push(
            &format!("{type_} {name}", type_ = r#type.to_string(), name = name),
            indent,
        );
    } else {
        out.push("query", indent);
    }
    format_selection_set(r#type.selection_set().clone(), indent, out);
    out.push_str("\n");
}

enum OperationType {
    Query(Query),
    Mutation(Mutation),
    Subscription(Subscription),
}

impl OperationType {
    fn selection_set(&self) -> &SelectionSet {
        match self {
            OperationType::Query(q) => &q.selection_set,
            OperationType::Mutation(m) => &m.selection_set,
            OperationType::Subscription(s) => &s.selection_set,
        }
    }

    fn name(&self) -> &Option<String> {
        match self {
            OperationType::Query(x) => &x.name,
            OperationType::Mutation(x) => &x.name,
            OperationType::Subscription(x) => &x.name,
        }
    }

    fn directives(&self) -> &Vec<Directive> {
        match self {
            OperationType::Query(x) => &x.directives,
            OperationType::Mutation(x) => &x.directives,
            OperationType::Subscription(x) => &x.directives,
        }
    }

    fn variable_definitions(&self) -> &Vec<VariableDefinition> {
        match self {
            OperationType::Query(x) => &x.variable_definitions,
            OperationType::Mutation(x) => &x.variable_definitions,
            OperationType::Subscription(x) => &x.variable_definitions,
        }
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OperationType::Query(_) => write!(f, "query"),
            OperationType::Mutation(_) => write!(f, "mutation"),
            OperationType::Subscription(_) => write!(f, "subscription"),
        }
    }
}

fn format_selection_set(set: SelectionSet, indent: &mut Indentation, out: &mut Output) {
    let mut items = set.items;

    if items.is_empty() {
        return;
    }

    out.push_str(" {\n");
    indent.increment();

    items.sort_unstable_by_key(selection_set_sort_key);

    for selection in items {
        match selection {
            Selection::Field(field) => format_field(field, indent, out),
            Selection::FragmentSpread(frag_spread) => {
                todo_field!(frag_spread.directives);
                out.push(&format!("...{}\n", frag_spread.fragment_name), indent);
            }
            Selection::InlineFragment(inline_frag) => todo!("inline_frag"),
        }
    }
    indent.decrement();
    out.push("}\n", indent);
}

fn selection_set_sort_key(sel: &Selection) -> (usize, String) {
    match sel {
        Selection::FragmentSpread(frag_spread) => (1, frag_spread.fragment_name.clone()),
        Selection::InlineFragment(inline_frag) => {
            if let Some(TypeCondition::On(ref name)) = inline_frag.type_condition {
                (2, name.clone())
            } else {
                (2, "zzzzz".to_string())
            }
        }
        Selection::Field(field) => {
            if field.selection_set.items.is_empty() {
                (3, field.name.clone())
            } else {
                (4, field.name.clone())
            }
        }
    }
}

fn format_field(field: Field, indent: &mut Indentation, out: &mut Output) {
    todo_field!(field.directives);

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

        let mut args = field
            .arguments
            .iter()
            .map(|(key, value)| format!("{arg}: {value}", arg = key, value = value.to_string()))
            .collect::<Vec<_>>();
        args.sort_unstable();
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
  country {
    id
  }
  team {
    id
    slug
    league {
      id
    }
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

    #[test]
    fn mutation() {
        let query = "
mutation NewUser {
  newUser(name: \"Bob\") { id }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
mutation NewUser {
  newUser(name: \"Bob\") {
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
    fn sorting_args() {
        let query = "
query UserProfile {
  user(x: 1, h: 1, a: 1) {
    team
  }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query UserProfile {
  user(a: 1, h: 1, x: 1) {
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
