use crate::{
    error::{CompilerError, Result},
    parsed,
};

#[derive(Debug)]
pub struct InkParser<'source> {
    input_string: &'source str,
    source_filename: Option<&'source str>,
}

impl<'source> InkParser<'source> {
    pub fn new(input_string: &'source str, source_filename: Option<&'source str>) -> Self {
        Self {
            input_string,
            source_filename,
        }
    }

    pub fn input_string(&self) -> &'source str {
        self.input_string
    }

    pub fn source_filename(&self) -> Option<&'source str> {
        self.source_filename
    }

    pub fn parse(&mut self) -> Result<parsed::Story> {
        Err(CompilerError::Unsupported(
            "InkParser has not been ported from ink-csharp/compiler/InkParser yet",
        ))
    }
}
