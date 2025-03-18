use std::num::NonZero;

use rusqlite::ffi::FTS5_TOKENIZE_DOCUMENT;

use super::{Token, TokenizerImpl};

pub struct Jieba {
    jieba: jieba::Jieba,
}

impl TokenizerImpl for Jieba {
    fn new(_args: &[&[u8]]) -> Result<Self, std::num::NonZero<i32>> {
        // TODO: support load dict

        #[cfg(feature = "jieba-default-dict")]
        let jieba = jieba::Jieba::new();
        #[cfg(not(feature = "jieba-default-dict"))]
        let jieba = jieba::Jieba::empty();
        Ok(Self { jieba })
    }

    fn tokenize(&self, text: &str, _flags: i32) -> Result<Vec<Token>, NonZero<i32>> {
        let jieba = &self.jieba;
        let is_document = _flags & FTS5_TOKENIZE_DOCUMENT as i32 != 0;
        let tokens = if is_document {
            jieba.tokenize(text, jieba::TokenizeMode::Search, false)
        } else {
            jieba.tokenize(text, jieba::TokenizeMode::Default, false)
        };

        let tokens = tokens.into_iter().map(|t| Token {
            text: t.word.to_string(),
            start: t.start,
            end: t.end,
            colocated: false,
        });

        Ok(tokens.collect())
    }
}
