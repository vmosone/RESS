use combine::{
    choice, eof,
    error::ParseError,
    parser::{char::string, repeat::take_until},
    try, Parser, Stream,
};
use strings;
use tokens::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct Comment {
    pub kind: Kind,
    pub content: String,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    Single,
    Multi,
}

impl Comment {
    pub fn from_parts(content: String, kind: Kind) -> Self {
        Comment { content, kind }
    }

    pub fn is_multi_line(&self) -> bool {
        self.kind == Kind::Multi
    }

    pub fn is_single_line(&self) -> bool {
        self.kind == Kind::Single
    }
}

impl ToString for Comment {
    fn to_string(&self) -> String {
        match self.kind {
            Kind::Single => format!("//{}", self.content),
            Kind::Multi => format!("/*{}*/", self.content),
        }
    }
}

pub(crate) fn comment<I>() -> impl Parser<Input = I, Output = Token>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (choice((try(multi_comment()), try(single_comment()))).map(|t: Comment| Token::Comment(t)))
}

pub(crate) fn single_comment<I>() -> impl Parser<Input = I, Output = Comment>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("//"),
        take_until(choice((
            try(strings::line_terminator_sequence()),
            try(eof().map(|_| String::new())),
        ))),
    )
        .map(|(_, content): (_, String)| Comment::from_parts(content, Kind::Single))
}

pub(crate) fn multi_comment<I>() -> impl Parser<Input = I, Output = Comment>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        multi_line_comment_start(),
        take_until(try(string("*/"))),
        multi_line_comment_end(),
    )
        .map(|(_s, c, _e): (String, String, String)| Comment::from_parts(c, Kind::Multi))
}

fn multi_line_comment_start<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (string("/*")).map(|s| s.to_string())
}

fn multi_line_comment_end<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (string("*/")).map(|s| s.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use tokens::token;
    #[test]
    fn comments_test() {
        let tests = vec![
            "//single line comments",
            "// another one with a space",
            "/*inline multi comments*/",
            "/*multi line comments
* that have extra decoration
* to help with readability
*/",
        ];
        for test in tests {
            let is_multi = test.starts_with("/*");
            let p = token().parse(test.clone()).unwrap();
            let comment_contents = test
                .lines()
                .map(|l| {
                    l.trim()
                        .replace("//", "")
                        .replace("/*", "")
                        .replace("*/", "")
                }).collect::<Vec<String>>()
                .join("\n");
            assert_eq!(p, (Token::comment(&comment_contents, is_multi), ""));
        }
    }
    proptest!{
        #[test]
        fn comments_prop(s in r#"((//.*)+|(/\*(.+[\n\r*])+\*/))"#) {
            let r = token().easy_parse(s.as_str()).unwrap();
            assert!(r.0.is_comment(), r.0.matches_comment_str(&format_test_comment(&s)));
        }
    }

    fn format_test_comment(s: &str) -> String {
        s.lines()
            .map(|l| {
                l.trim()
                    .trim_left_matches("//")
                    .trim_left_matches("/*")
                    .trim_right_matches("*/")
            }).collect::<Vec<&str>>()
            .join("\n")
    }
}
