# Serdoc

An experiment to serialize Rust structs to TOML, while preserving field docstrings.

See also:
* https://github.com/serde-rs/serde/issues/1430
* https://github.com/toml-rs/toml-rs/issues/274
* https://github.com/toml-rs/toml/issues/376

# Example

```rust
use documented::DocumentedFields;
use serde::Serialize;
use struct_field_names_as_array::FieldNamesAsArray;
use toml_edit::ser::to_document;

/// A test struct
#[derive(Default, DocumentedFields, FieldNamesAsArray, Serialize)]
struct TestMe {
    /// First field
    ///
    /// Second line
    field1: u32,
    /// Second field
    field2: String,
}

let test = TestMe::default();
let mut doc = to_document(&test).unwrap();

for (mut key, _value) in doc.iter_mut() {
    let key_name = key.get().to_owned();
    let decor = key.decor_mut();
    let docstring = TestMe::get_field_comment(key_name).unwrap();

    let mut comment = String::new();
    for line in docstring.lines() {
        let line = if line.is_empty() {
            String::from("#\n")
        } else {
            format!("# {line}\n")
        };
        comment.push_str(&line);
    }
    decor.set_prefix(comment);
}

let actual = doc.to_string();
// There's a bug in rustdoc that eats the leading '#' from a multiline string, so collapse onto a
// single line
let expected = "# First field\n#\n# Second line\nfield1 = 0\n# Second field\nfield2 = \"\"\n";
assert_eq!(actual, expected);
```
