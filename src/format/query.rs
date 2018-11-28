use super::{Indentation, Output, INDENT_SIZE, MAX_LINE_LENGTH};
use failure::{bail, Error};
use graphql_parser::{parse_query, query::*};

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
        Definition::Fragment(fragment) => format_fragment(fragment, indent, out),
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

fn format_fragment(frag: FragmentDefinition, indent: &mut Indentation, out: &mut Output) {
    out.push(
        &format!(
            "fragment {name} {type_}",
            name = frag.name,
            type_ = frag.type_condition
        ),
        indent,
    );
    format_selection_set(frag.selection_set, indent, out);
}

fn format_operation_type(r#type: OperationType, indent: &mut Indentation, out: &mut Output) {
    todo_field!(r#type.directives());

    let has_name;
    if let Some(name) = r#type.name() {
        has_name = true;
        out.push(
            &format!("{type_} {name}", type_ = r#type.to_string(), name = name),
            indent,
        );
    } else {
        has_name = false;
        out.push(&format!("{type_}", type_ = r#type.to_string()), indent);
    }

    if !r#type.variable_definitions().is_empty() {
        if has_name {
            out.push_str("(");
        } else {
            out.push_str(" (");
        }
        let args = r#type
            .variable_definitions()
            .iter()
            .map(|var| {
                let mut out = format!("${name}: {type_}", name = var.name, type_ = var.var_type);
                if let Some(default) = &var.default_value {
                    out.push_str(&format!(" = {}", default));
                }
                out
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("{}", args));
        out.push_str(")");
    }

    format_selection_set(r#type.selection_set().clone(), indent, out);
    out.push_str("\n");
}

enum OperationType {
    Query(Query),
    Mutation(Mutation),
    Subscription(Subscription),
}

macro_rules! get_operation_type_field {
    ($self:ident, $field:ident) => {
        match $self {
            OperationType::Query(x) => &x.$field,
            OperationType::Mutation(x) => &x.$field,
            OperationType::Subscription(x) => &x.$field,
        }
    };
}

impl OperationType {
    fn selection_set(&self) -> &SelectionSet {
        get_operation_type_field!(self, selection_set)
    }

    fn name(&self) -> &Option<String> {
        match self {
            OperationType::Query(x) => &x.name,
            OperationType::Mutation(x) => &x.name,
            OperationType::Subscription(x) => &x.name,
        }
    }

    fn directives(&self) -> &Vec<Directive> {
        get_operation_type_field!(self, directives)
    }

    fn variable_definitions(&self) -> &Vec<VariableDefinition> {
        get_operation_type_field!(self, variable_definitions)
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
            Selection::InlineFragment(inline_frag) => {
                format_inline_fragment(inline_frag, indent, out)
            }
        }
    }
    indent.decrement();
    out.push("}\n", indent);
}

fn selection_set_sort_key(sel: &Selection) -> (usize, String) {
    match sel {
        Selection::FragmentSpread(frag_spread) => (3, frag_spread.fragment_name.clone()),
        Selection::InlineFragment(inline_frag) => {
            if let Some(TypeCondition::On(ref name)) = inline_frag.type_condition {
                (4, name.clone())
            } else {
                (5, String::new())
            }
        }
        Selection::Field(field) => {
            if field.selection_set.items.is_empty() {
                (1, field.name.clone())
            } else {
                (2, field.name.clone())
            }
        }
    }
}

fn format_inline_fragment(inline_frag: InlineFragment, indent: &mut Indentation, out: &mut Output) {
    todo_field!(inline_frag.directives);

    out.push("...", indent);
    if let Some(TypeCondition::On(type_condition)) = inline_frag.type_condition {
        out.push_str(&format!(" on {}", type_condition));
    }
    format_selection_set(inline_frag.selection_set, indent, out);
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
    use crate::format::format_test;
    use super::*;

    #[test]
    fn test_basic() {
        format_test(
            format,
            "
query One { firstName }
query Two { firstName lastName }
            ",
            "
query One {
  firstName
}

query Two {
  firstName
  lastName
}
            ",
        );
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

    #[test]
    fn variables() {
        let query = "
query UserProfile ($username:String!
) {
  reddit {
    user(username: $username) {
      username
      commentKarma
      createdISO
    }
  }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query UserProfile($username: String!) {
  reddit {
    user(username: $username) {
      commentKarma
      createdISO
      username
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
    fn variables_with_default_args() {
        let query = "
query UserProfile ($username:String! = \"123\"
) {
  reddit {
    user(username: $username) {
      username
      commentKarma
      createdISO
    }
  }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query UserProfile($username: String! = \"123\") {
  reddit {
    user(username: $username) {
      commentKarma
      createdISO
      username
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
    fn unnamed_query_with_variables() {
        let query = "
query   ($username:String!
) {
  reddit {
    user(username: $username) {
      username
      commentKarma
      createdISO
    }
  }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query ($username: String!) {
  reddit {
    user(username: $username) {
      commentKarma
      createdISO
      username
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
    fn inline_fragment() {
        let query = "
query {
  a(query:\"hi\") {
    ... on User {
      id
      name
    }
  }
  b(query: \"hi\") {
    ... {
      id
      name
    }
  }
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
query {
  a(query: \"hi\") {
    ... on User {
      id
      name
    }
  }
  b(query: \"hi\") {
    ... {
      id
      name
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
    fn fragment_definition() {
        let query = "
fragment comparisonFields on Character {
name
friendsConnection(first: $first) {
  totalCount
  edges {
    node {
      name
    }
  }
}
}
        "
        .trim();

        let actual = format(query).unwrap();
        let expected = "
fragment comparisonFields on Character {
  name
  friendsConnection(first: $first) {
    totalCount
    edges {
      node {
        name
      }
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
}
