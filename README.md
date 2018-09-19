# RESS
> Rusty EcmaScript Scanner

[![travis](https://img.shields.io/travis/FreeMasen/RESS.svg)](https://travis-ci.org/FreeMasen/RESS)
[![appveyor](https://img.shields.io/appveyor/ci/FreeMasen/RESS.svg)](https://ci.appveyor.com/project/FreeMasen/sitebuilder)
[![crates.io](https://img.shields.io/crates/v/ress.svg)](https://crates.io/crates/ress)
[![last commit master](https://img.shields.io/github/last-commit/FreeMasen/RESS.svg)](https://github.com/FreeMasen/RESS/commits/master)

A scanner/tokenizer for JS written in Rust

## Usage
There are two main interfaces for using `ress` in your Rust code.

The first is the very simple function `tokenize`, this takes in a `String` and outputs a `Vec<Token>`.

```rust
extern crate ress;

use ress::tokenize;

static &str JS = include_str!("index.js");

fn main() {
    let tokens = tokenize(JS);
    it !tokens.iter().any(|t| t.is_punct_with(";")) {
        panic!("No semi-colon!? You nave!");
    } else {
        println!("At least you are sane at one point");
    }
}

```

The other option is to create a `Scanner`, an iterator over the `Item` struct. `Item` has two fields `token` for the `Token` found and `Span` for the position in the string.
```rust
extern crate ress;

use ress::{Scanner};

const &str JS = include_str!("index.js");

fn main() {
    let s = Scanner::new(JS);
    for token in s {
        if token.is_punct_with(";") {
            panic!("A semi-colon!? Heathen!");
        }
    }
    println!("Good show! Why use something that's optional?")
}
```

In either method the major construct that you would be dealing with is a `Token` enum. This enum represents the 10 different tokens defined in the ECMAScript specification.

### ES Tokens
- Boolean Literal
- End of File
- Identifier
- Keyword
- Null Literal
- Numeric Literal
- Punctuation
- String Literal
- Regular Expression Literal
- Comment

In its current state it should be able to tokenize any valid JavaScript (I believe the testing is all currently done on ES3 packages). Keep in mind that keywords have been moving around a lot in JS between ES3 through ES2019 so you might find some items parsed as keywords in the ES2019 context that are not in the ES3 context and since my goal is keep this scanner context free this should be dealt with at a higher level. A good example of this is `yield` which is sometimes a keyword and sometimes an identifier, this package will always parse this as a Keyword.

For each of the token cases there is either a struct or enum to provide additional information with the exception of `NullLiteral` and `EoF` which should be self explanatory. The more complicated items do implement `ToString` which should get you back to the original js text for that token. The `Token` enum also provides a number of helper functions for building that picture without pulling the inner data our of the enum. Using the `Punct` case as an example the helper functions look like this
```rust
fn is_punct(&self) -> bool;
fn matches_punct(&self, p: Punct) -> bool;
fn matches_punct_str(&self, s: &str) -> bool;
```
A similar set of functions are available for each case. Be aware that some `_str` implementations panic if the wrong string is provided meaning these would also panic.
```rust
let p = Token::Punct(Keyword::This);
if p.matches_keyword_str("junk") {
    // panic!
}
if p.matches_keyword(Keyword::This) {
    // Don't panic!
}
```

Like all `Iterators` the `Scanner` has a `next`, I have also implemented a `look_ahead` method that will allow you to parse the next value without advancing. Using this method can be a convenient way to get the next token without performing a mutable borrow, however you will be incurring the cost of parsing that token twice. All `Iterators` implement `Peekable` that will convert them into a new iterator with a `peek` method, this will allow you to look ahead while only paying the cost once however `peek` performs a mutable borrow which means it needs to be in a different scope than a call to `next`.
```rust
// look_ahead
let js = "function() { return; }";
let mut s = Scanner::new(js);
let current = s.next();
let next = s.look_ahead();
let new_current = s.next();
assert_eq!(next, new_current);
// peekable (fails to compile)
let p = Scanner::new(js).peekable();
let current = s.next(); // <-- first mutable borrow
let next = p.peek(); // <-- second mutable borrow
```

For more intense lookahead scenarios `Scanner` makes available the `get_state` and `set_state` methods. These methods will allow you to capture a snapshot of the current position and any context, and then later reset to that position and context.

```rust
let js = "function() {
    return 0;
};";
let mut s = Scanner::new(js);
let start = s.get_state();
assert_eq!(s.next().unwrap().token, Token::Keyword(Keyword::Function));
assert_eq!(s.next().unwrap().token, Token::Punct(Punct::OpenParen));
assert_eq!(s.next().unwrap().token, Token::Punct(Punct::CloseParen));
s.set_state(start);
assert_eq!(s.next().unwrap().token, Token::Keyword(Keyword::Function));
```

## Why?
Wouldn't it be nice to write new JS development tools in Rust? The (clear-comments)[https://github.com/FreeMasen/RESS/blob/master/examples/clear-comments/src/main.rs] example is a proof of concept on how you might use this crate to do just that. This example will take in a JS file and output a version with all of the comments removed. An example of how you might see it in action is below (assuming you have a file called in.js in the project root).
```sh
$ cargo run --example clear-comments -- ./in.js ./out.js
```

Ideally this project will be the starting point for building a full JS Abstract Syntax Tree (AST) in Rust. The next step would be to build a companion crate that will raise the tokens into a full (AST) ([work in progress](https://github.com/freemasen/resp)). And once we have an AST building JS dev tools in rust would be significantly easier.

# Performance
I am sure there are a lot of low hanging fruit in this area.
The below stats are from running `cargo +nightly bench` on a Dell XPS 13 9350 (2.3ghz i5-6200U & 8bg RAM).

|Lib           |Size     |Time     |+/-     |
|---           |---      |---      |---     |
|Angular 1.5.6 |1.16mb   |333.33ms |17.29ms |
|jquery        |271.75kb |171.48ms |73.16ms |
|React         |59.09kb  | 24.60ms | 4,83ms |
|React-dom     |641.51kb |288.60ms |25.61ms |
|Vue           |289.30kb |187.04ms |60.05ms |

If you are interested in getting an idea about performance without waiting for `cargo bench` to complete you can run the following command.

```sh
$ cargo run --example major_libs
```

# Contributing

[see contributing.md](https://github.com/FreeMasen/RESS/blob/master/CONTRIBUTING.md)