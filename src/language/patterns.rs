use crate::language::symbols::KEYWORDS;
use crate::utils::parsing::{Pattern, PatternPart};

pub(crate) const IDENTIFIER_PATTERN: Pattern = Pattern {
    name: "identifier",
    excluded_tokens: KEYWORDS,
    parts: &[
        PatternPart {
            is_valid_char: |char| char.is_ascii_alphabetic() || char == '_',
            min_count: 1,
            max_count: 1,
        },
        PatternPart {
            is_valid_char: |char| char.is_ascii_alphanumeric() || char == '_',
            min_count: 0,
            max_count: usize::MAX,
        },
    ],
};

pub(crate) const I32_LITERAL_PATTERN: Pattern = Pattern {
    name: "`i32` literal",
    excluded_tokens: &[],
    parts: &[
        PatternPart {
            is_valid_char: |char| char == '-',
            min_count: 0,
            max_count: 1,
        },
        PatternPart {
            is_valid_char: |char| char.is_ascii_digit(),
            min_count: 1,
            max_count: 1,
        },
        PatternPart {
            is_valid_char: |char| char.is_ascii_digit() || char == '_',
            min_count: 0,
            max_count: usize::MAX,
        },
    ],
};
