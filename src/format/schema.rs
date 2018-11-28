use super::{map_join, Indentation, Output, INDENT_SIZE, MAX_LINE_LENGTH};
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
            // TODO: directives

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

        Definition::TypeExtension(_) => unimplemented!("TypeExtension"),

        Definition::DirectiveDefinition(_) => unimplemented!("DirectiveDefinition"),
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
                map_join(obj.implements_interfaces.iter(), |name| name, " & ", out);
            }

            out.push_str(" {\n");
            format_fields(obj.fields, indent, out);
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

        TypeDefinition::Interface(interface) => {
            // TODO: description
            // TODO: directives

            out.push(
                &format!("interface {name} {{\n", name = interface.name),
                indent,
            );
            format_fields(interface.fields, indent, out);
            out.push("}\n\n", indent);
        }

        TypeDefinition::InputObject(obj) => {
            // TODO: description
            // TODO: directives

            out.push(&format!("input {name} {{\n", name = obj.name), indent);
            format_input_values(obj.fields, indent, out);
            out.push("}\n\n", indent);
        }

        TypeDefinition::Union(union) => {
            // TODO: description
            // TODO: directives

            out.push(&format!("union {name} = ", name = union.name), indent);

            let mut types = union.types;
            types.sort_unstable_by_key(|type_| type_.clone());
            map_join(types.iter(), |type_| type_, " | ", out);
            out.push_str("\n\n");
        }
    }
}

fn format_fields(fields: Vec<Field>, indent: &mut Indentation, out: &mut Output) {
    indent.increment();

    let mut fields = fields.clone();
    fields.sort_unstable_by_key(|field| field.name.clone());

    for field in fields {
        format_field(field, indent, out);
    }

    indent.decrement();
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

fn format_input_values(values: Vec<InputValue>, indent: &mut Indentation, out: &mut Output) {
    indent.increment();

    let mut values = values.clone();
    values.sort_unstable_by_key(|field| field.name.clone());

    for value in values {
        format_input_value(value, indent, out);
    }

    indent.decrement();
}

fn format_input_value(value: InputValue, indent: &mut Indentation, out: &mut Output) {
    // TODO: description
    // TODO: default value
    // TODO: directives

    out.push(
        &format!(
            "{name}: {type_}\n",
            name = value.name,
            type_ = value.value_type
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
    fn test_implements_interfaces() {
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

    #[test]
    fn test_define_interface() {
        format_test(
            format,
            "
interface Character { id: ID! appearsIn: [Episode]! friends: [Character] name: String! }
            ",
            "
interface Character {
  appearsIn: [Episode]!
  friends: [Character]
  id: ID!
  name: String!
}
            ",
        );
    }

    #[test]
    fn test_union() {
        format_test(
            format,
            "
union SearchResult
    = Z | Human | Droid | Starship
            ",
            "
union SearchResult = Droid | Human | Starship | Z
            ",
        );
    }
}
