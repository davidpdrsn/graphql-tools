use super::{Indentation, Output, INDENT_SIZE, MAX_LINE_LENGTH};
use failure::{bail, Error};
use graphql_parser::parse_schema;
use graphql_parser::schema::*;

pub fn format(contents: &str) -> Result<String, Error> {
    let ast = parse_schema(contents)?;

    let mut out = Output::new();
    let mut indent = Indentation::new(INDENT_SIZE);

    for def in ast.definitions {
        format_def(def, &mut indent, &mut out);
    }

    Ok(out.trim().to_string())
}

fn format_def(def: Definition, indent: &mut Indentation, out: &mut Output) {
    match def {
        Definition::SchemaDefinition(schema_def) => {
            todo_field!(schema_def.directives);

            out.push("schema {\n", indent);
            indent.increment();
            if let Some(mutation) = schema_def.mutation {
                out.push(&format!("mutation: {}\n", mutation), indent);
            }
            if let Some(query) = schema_def.query {
                out.push(&format!("query: {}\n", query), indent);
            }
            if let Some(subscription) = schema_def.subscription {
                out.push(&format!("subscription: {}\n", subscription), indent);
            }
            indent.decrement();
            out.push("}\n\n", indent);
        }

        Definition::TypeDefinition(type_def) => format_type(type_def, indent, out),

        Definition::TypeExtension(_) => todo!("TypeExtension"),

        Definition::DirectiveDefinition(_) => todo!("DirectiveDefinition"),
    }
}

fn format_type(type_def: TypeDefinition, indent: &mut Indentation, out: &mut Output) {
    match type_def {
        TypeDefinition::Object(obj) => {
            // TODO: description
            // TODO: directives

            out.push(&format!("type {name}", name = obj.name), indent);
            if !obj.implements_interfaces.is_empty() {
                out.push_str(" implements ");
                let interfaces = obj
                    .implements_interfaces
                    .iter()
                    .map(|name| format!("{}", name))
                    .collect::<Vec<_>>()
                    .join(" & ");
                out.push_str(&interfaces);
            }
            out.push_str(" {\n");

            indent.increment();
            let mut fields = obj.fields;
            fields.sort_unstable_by_key(|field| field.name.clone());
            for field in fields {
                format_field(field, indent, out);
            }
            indent.decrement();

            out.push("}\n\n", indent);
        }

        TypeDefinition::Enum(enum_) => {
            // TODO: description
            // TODO: directives

            out.push(&format!("enum {name} {{\n", name = enum_.name), indent);
            indent.increment();
            let mut values = enum_.values;
            values.sort_unstable_by_key(|field| field.name.clone());
            for value in values {
                out.push(&format!("{name}\n", name = value.name), indent);
            }
            indent.decrement();

            out.push("}\n\n", indent);
        }

        TypeDefinition::Scalar(scalar) => {
            // TODO: description
            // TODO: directives

            out.push(&format!("scalar {name}\n\n", name = scalar.name), indent);
        }

        TypeDefinition::Interface(_) => todo!("Interface"),
        TypeDefinition::Union(_) => todo!("Union"),
        TypeDefinition::InputObject(_) => todo!("InputObject"),
    }
}

fn format_field(field: Field, indent: &mut Indentation, out: &mut Output) {
    // TODO: description
    // TODO: name
    // TODO: arguments
    // TODO: directives

    out.push(
        &format!(
            "{name}: {type_}\n",
            name = field.name,
            type_ = field.field_type
        ),
        indent,
    );
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use crate::format::format_test;

    #[test]
    fn test_basic() {
        format_test(
            format,
            "
type User { id: Int! name: String }
schema { query:Query mutation:Mutation }
            ",
            "
type User {
  id: Int!
  name: String
}

schema {
  mutation: Mutation
  query: Query
}
            ",
        );
    }

    #[test]
    fn test_schema_first() {
        format_test(
            format,
            "
schema { query:Query mutation:Mutation }
type User { id: Int! name: String }
            ",
            "
schema {
  mutation: Mutation
  query: Query
}

type User {
  id: Int!
  name: String
}
            ",
        );
    }

    #[test]
    fn test_interfaces() {
        format_test(
            format,
            "
type User implements Foo & Bar & Baz { id: Int! name: String }
type Team implements Foo { id: Int! name: String }
            ",
            "
type User implements Foo & Bar & Baz {
  id: Int!
  name: String
}

type Team implements Foo {
  id: Int!
  name: String
}
            ",
        );
    }

    #[test]
    fn test_enum() {
        format_test(
            format,
            "
enum Number { ONE TWO THREE }
            ",
            "
enum Number {
  ONE
  THREE
  TWO
}
            ",
        );
    }

    #[test]
    fn test_scalar() {
        format_test(
            format,
            "
            scalar DateTime
            ",
            "
scalar DateTime
            ",
        );
    }
}
