# SDLang

An [SDLang][sdlang] parser library.

[SDLang][sdlang] is a simple and concise way to textually represent data.
It has an XML-like structure - tags, values, and attributes - which makes
it a versatily choice for data serialization, configuration files, or
declarative languages. Its syntax was inspired by the C family of languages
(C/C++, C#, D, Java, ...).

Here's an example from the official website:
```sdlang
// This is a node with a single string value
title "Hello, World"

// Multiple values are supported, too
bookmarks 12 15 188 1234

// Nodes can have attributes
author "Peter Parker" email="peter@example.org" active=true

// Nodes can be arbitrarily nested
contents {
    section "First Section" {
        paragraph "This is the first paragraph"
        paragraph "This is the second paragraph"
    }
}

// Anonymous nodes are supported
"This text is the value of an anonymous node!"

// This makes things like matrix definiotns very convenient
matrix {
    1 0 0
    0 1 0
    0 0 1
}
```

Parsing is made as easy as this:
```rust
extern crate sdlang;

// Prints `tag hello_world: "text"`
println!("{}", sdlang::parse_text("hello_world \"text\"").unwrap());
```

[sdlang]: https://sdlang.org "Official SDLang Website"
