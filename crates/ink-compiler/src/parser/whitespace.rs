use super::string_parser::{
    inline_whitespace_characters, ParseSuccessStruct, StringParser, PARSE_SUCCESS,
};

pub fn newline(parser: &mut StringParser) -> Option<ParseSuccessStruct> {
    whitespace(parser);

    parser.parse_newline().map(|_| PARSE_SUCCESS)
}

pub fn end_of_file(parser: &mut StringParser) -> Option<ParseSuccessStruct> {
    whitespace(parser);

    if parser.end_of_input() {
        Some(PARSE_SUCCESS)
    } else {
        None
    }
}

pub fn end_of_line(parser: &mut StringParser) -> Option<ParseSuccessStruct> {
    newline(parser).or_else(|| end_of_file(parser))
}

pub fn whitespace(parser: &mut StringParser) -> Option<ParseSuccessStruct> {
    if parser
        .parse_characters_from_char_set(&inline_whitespace_characters(), true, None)
        .is_some()
    {
        Some(PARSE_SUCCESS)
    } else {
        None
    }
}

pub fn multiline_whitespace(parser: &mut StringParser) -> Option<ParseSuccessStruct> {
    let mut found_any = false;

    while newline(parser).is_some() {
        found_any = true;
    }

    if found_any {
        Some(PARSE_SUCCESS)
    } else {
        None
    }
}

pub fn any_whitespace(parser: &mut StringParser) -> Option<ParseSuccessStruct> {
    let mut found_any = false;

    while whitespace(parser).is_some() || multiline_whitespace(parser).is_some() {
        found_any = true;
    }

    if found_any {
        Some(PARSE_SUCCESS)
    } else {
        None
    }
}

pub fn spaced<T, F>(parser: &mut StringParser, mut rule: F) -> Option<T>
where
    F: FnMut(&mut StringParser) -> Option<T>,
{
    whitespace(parser);

    let result = parser.parse_object(|parser| rule(parser))?;

    whitespace(parser);

    Some(result)
}

pub fn multi_spaced<T, F>(parser: &mut StringParser, mut rule: F) -> Option<T>
where
    F: FnMut(&mut StringParser) -> Option<T>,
{
    any_whitespace(parser);

    let result = parser.parse_object(|parser| rule(parser))?;

    any_whitespace(parser);

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::{
        any_whitespace, end_of_file, end_of_line, multiline_whitespace, newline, spaced, whitespace,
    };
    use crate::parser::StringParser;

    #[test]
    fn whitespace_helpers_handle_inline_and_line_breaks() {
        let mut parser = StringParser::new(" \t\nx");

        assert!(whitespace(&mut parser).is_some());
        assert!(newline(&mut parser).is_some());
        assert_eq!(parser.current_character(), 'x');
    }

    #[test]
    fn multiline_and_any_whitespace_consume_multiple_newlines() {
        let mut multiline_parser = StringParser::new("\n\nx");
        let mut any_parser = StringParser::new(" \r\nx");

        assert!(multiline_whitespace(&mut multiline_parser).is_some());
        assert_eq!(multiline_parser.current_character(), 'x');

        assert!(any_whitespace(&mut any_parser).is_some());
        assert_eq!(any_parser.current_character(), 'x');
    }

    #[test]
    fn end_of_line_accepts_newline_or_eof() {
        let mut newline_parser = StringParser::new("  \n");
        let mut eof_parser = StringParser::new("  ");

        assert!(end_of_line(&mut newline_parser).is_some());
        assert!(end_of_file(&mut eof_parser).is_some());
    }

    #[test]
    fn spaced_wraps_a_parse_rule() {
        let mut parser = StringParser::new("  hello  ");

        let parsed = spaced(&mut parser, |parser| parser.parse_string("hello"));
        assert_eq!(parsed, Some("hello".to_string()));
        assert!(parser.end_of_input());
    }
}
