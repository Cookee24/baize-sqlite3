use std::num::NonZero;

use super::TokenizerImpl;

pub struct Simple {}

impl TokenizerImpl for Simple {
    fn new(_args: &[&[u8]]) -> Result<Self, NonZero<i32>> {
        Ok(Self {})
    }

    fn tokenize(&self, text: &str, _flags: i32) -> Result<Vec<super::Token>, NonZero<i32>> {
        let mut tokens = Vec::new();
        let mut iter = text.char_indices().peekable();
        while let Some((idx, char)) = iter.next() {
            let cur_type = TokenType::from(char);

            match cur_type {
                TokenType::WhiteSpace | TokenType::Delimiter => {}
                TokenType::Numeric => {
                    let mut end_idx = idx + char.len_utf8();
                    let mut num_str = char.to_string();

                    while let Some((next_idx, next_char)) = iter.peek().cloned() {
                        let next_type = TokenType::from(next_char);

                        // Dealing with the case like "1,000.00"
                        if next_type == TokenType::Numeric
                            || (next_char == '.' || next_char == ',')
                                && iter.clone().nth(1).map_or(false, |(_, c)| {
                                    TokenType::from(c) == TokenType::Numeric
                                })
                        {
                            num_str.push(next_char);
                            end_idx = next_idx + next_char.len_utf8();
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    let clean_num = num_str.replace(',', "");

                    tokens.push(super::Token {
                        text: clean_num,
                        start: idx,
                        end: end_idx,
                        colocated: false,
                    });
                }
                TokenType::Other => {
                    let len = char.len_utf8();
                    tokens.push(super::Token {
                        text: char.to_string(),
                        start: idx,
                        end: idx + len,
                        colocated: false,
                    });
                }
                _ => {
                    let mut end_idx = idx + char.len_utf8();
                    let mut end_pos = end_idx;

                    while let Some((next_idx, next_char)) = iter.peek().cloned() {
                        if TokenType::from(next_char) != cur_type {
                            break;
                        }
                        end_pos = next_idx + next_char.len_utf8();
                        end_idx = end_pos;
                        iter.next();
                    }

                    #[cfg(not(feature = "porter"))]
                    let token_text = if cur_type == TokenType::Alpha {
                        text[idx..end_pos].to_ascii_lowercase()
                    } else {
                        text[idx..end_pos].to_string()
                    };

                    #[cfg(feature = "porter")]
                    let token_text = porter_stemmer::stem(&text[idx..end_pos].to_ascii_lowercase());

                    tokens.push(super::Token {
                        text: token_text,
                        start: idx,
                        end: end_idx,
                        colocated: false,
                    });
                }
            }
        }

        Ok(tokens)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TokenType {
    WhiteSpace,
    Delimiter,
    Alpha,
    Numeric,
    Other,
}

impl From<char> for TokenType {
    fn from(c: char) -> Self {
        if c.is_whitespace() {
            TokenType::WhiteSpace
        } else if c.is_ascii_punctuation() {
            TokenType::Delimiter
        } else if c.is_ascii_alphabetic() {
            TokenType::Alpha
        } else if c.is_ascii_digit() {
            TokenType::Numeric
        } else if is_chinese_delimiter(c) {
            TokenType::Delimiter
        } else {
            TokenType::Other
        }
    }
}

fn is_chinese_delimiter(c: char) -> bool {
    matches!(
        c,
        '。' | '，'
            | '、'
            | '；'
            | '：'
            | '？'
            | '！'
            | '“'
            | '”'
            | '‘'
            | '’'
            | '（'
            | '）'
            | '《'
            | '》'
            | '【'
            | '】'
            | '—'
            | '…'
    )
}

#[cfg(test)]
mod test {
    use crate::tokenize::TokenizerImpl;
    use sqlite3ext_sys::FTS5_TOKENIZE_DOCUMENT;

    use super::Simple;

    const TK: Simple = Simple {};

    macro_rules! test_doc {
        ($text:expr, $tokens:expr) => {
            let tokens = TK
                .tokenize($text, FTS5_TOKENIZE_DOCUMENT.try_into().unwrap())
                .unwrap()
                .into_iter()
                .map(|t| t.text)
                .collect::<Vec<_>>();

            assert_eq!(tokens, $tokens);
        };
    }

    #[test]
    fn test_zh() {
        test_doc!(
            "在美国叫超人，在中国叫电棍",
            vec![
                "在", "美", "国", "叫", "超", "人", "在", "中", "国", "叫", "电", "棍"
            ]
        );

        test_doc!(
            "   “”啊米浴说的道理、、",
            vec!["啊", "米", "浴", "说", "的", "道", "理"]
        );

        test_doc!("，。、；：？！“”‘’（）《》【】—…", Vec::<&str>::new());
    }

    #[test]
    fn test_en() {
        #[cfg(feature = "porter")]
        test_doc!(
            "The quick brown fox jumps over the lazy dog.",
            vec![
                "the", "quick", "brown", "fox", "jump", "over", "the", "lazi", "dog"
            ]
        );

        #[cfg(not(feature = "porter"))]
        test_doc!(
            "The quick brown fox jumps over the lazy dog.",
            vec![
                "the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"
            ]
        );
    }

    #[test]
    fn test_num() {
        test_doc!("1,000.00", vec!["1000.00"]);
        test_doc!("1,000", vec!["1000"]);
        test_doc!("1,000.00.00", vec!["1000.00.00"]);

        test_doc!("练习时长2.5年", vec!["练", "习", "时", "长", "2.5", "年"]);
    }

    #[test]
    fn test_mix() {
        test_doc!(
            "在美国叫超人Superman",
            vec!["在", "美", "国", "叫", "超", "人", "superman"]
        );

        test_doc!(
            "在美国叫超人Superman，中国叫电棍",
            vec![
                "在", "美", "国", "叫", "超", "人", "superman", "中", "国", "叫", "电", "棍"
            ]
        );
    }
}
