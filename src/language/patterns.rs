use crate::compiletools::parsing::{Pattern, PatternPart};
use crate::language::symbols::KEYWORDS;

pub(crate) const IDENT_PAT: Pattern = Pattern {
    name: "identifier",
    excluded_tokens: KEYWORDS,
    parts: &[
        PatternPart {
            is_valid_char: |c| c.is_ascii_alphabetic() || c == '_',
            min_count: 1,
            max_count: 1,
        },
        PatternPart {
            is_valid_char: |c| c.is_ascii_alphanumeric() || c == '_',
            min_count: 0,
            max_count: usize::MAX,
        },
    ],
};

pub(crate) const I32_LIT_PAT: Pattern = Pattern {
    name: "`i32` literal",
    excluded_tokens: &[],
    parts: &[
        PatternPart {
            is_valid_char: |c| c == '-',
            min_count: 0,
            max_count: 1,
        },
        PatternPart {
            is_valid_char: |c| c.is_ascii_digit(),
            min_count: 1,
            max_count: 1,
        },
        PatternPart {
            is_valid_char: |c| c.is_ascii_digit() || c == '_',
            min_count: 0,
            max_count: usize::MAX,
        },
    ],
};
