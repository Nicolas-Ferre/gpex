use convert_case::{Boundary, Case, Converter};

pub(crate) fn convert(name: &str, case: Case<'_>) -> String {
    let trimmed_name = name.trim_start_matches('_');
    let converter = Converter::new()
        .remove_boundaries(&Boundary::digits())
        .to_case(case);
    let converted_name = converter.convert(trimmed_name);
    let has_leading_underscore = name.len() - trimmed_name.len() > 0;
    let underscore_prefix = "_".repeat(has_leading_underscore.into());
    format!("{underscore_prefix}{converted_name}")
}

pub(crate) fn is_valid(name: &str, case: Case<'_>, converted_name: &str) -> bool {
    if case == Case::Pascal {
        let is_first_char_uppercase = name
            .strip_prefix('_')
            .unwrap_or(name)
            .chars()
            .next()
            .is_some_and(char::is_uppercase);
        is_first_char_uppercase && name.to_lowercase() == converted_name.to_lowercase()
    } else {
        converted_name == name
    }
}

#[cfg(test)]
mod tests {
    use super::{convert, is_valid};
    use convert_case::Case;

    #[test]
    fn convert_to_snake() {
        let case = Case::Snake;
        assert_eq!(convert("snake_case123", case), "snake_case123");
        assert_eq!(convert("snake_case_123", case), "snake_case_123");
        assert_eq!(convert("camelCase123", case), "camel_case123");
        assert_eq!(convert("PascalCase123", case), "pascal_case123");
        assert_eq!(convert("ACRONYMCase123", case), "acronym_case123");
        assert_eq!(convert("UPPER_CASE123", case), "upper_case123");
        assert_eq!(convert("_Underscore", case), "_underscore");
        assert_eq!(convert("__Underscores", case), "_underscores");
    }

    #[test]
    fn convert_to_upper() {
        let case = Case::UpperSnake;
        assert_eq!(convert("snake_case123", case), "SNAKE_CASE123");
        assert_eq!(convert("snake_case_123", case), "SNAKE_CASE_123");
        assert_eq!(convert("camelCase123", case), "CAMEL_CASE123");
        assert_eq!(convert("PascalCase123", case), "PASCAL_CASE123");
        assert_eq!(convert("ACRONYMCase123", case), "ACRONYM_CASE123");
        assert_eq!(convert("UPPER_CASE123", case), "UPPER_CASE123");
        assert_eq!(convert("_Underscore", case), "_UNDERSCORE");
        assert_eq!(convert("__Underscores", case), "_UNDERSCORES");
    }

    #[test]
    fn convert_to_pascal() {
        let case = Case::Pascal;
        assert_eq!(convert("snake_case123", case), "SnakeCase123");
        assert_eq!(convert("snake_case_123", case), "SnakeCase123");
        assert_eq!(convert("camelCase123", case), "CamelCase123");
        assert_eq!(convert("PascalCase123", case), "PascalCase123");
        assert_eq!(convert("ACRONYMCase123", case), "AcronymCase123");
        assert_eq!(convert("UPPER_CASE123", case), "UpperCase123");
        assert_eq!(convert("_underscore", case), "_Underscore");
        assert_eq!(convert("__underscores", case), "_Underscores");
    }

    #[test]
    fn validate_snake() {
        let case = Case::Snake;
        assert!(is_valid("snake_case123", case, "snake_case123"));
        assert!(is_valid("snake_case_123", case, "snake_case_123"));
        assert!(!is_valid("camelCase123", case, "camel_case123"));
        assert!(!is_valid("PascalCase123", case, "pascal_case123"));
        assert!(!is_valid("ACRONYMCase123", case, "acronym_case123"));
        assert!(!is_valid("UPPER_CASE123", case, "upper_case123"));
        assert!(is_valid("_underscore", case, "_underscore"));
        assert!(!is_valid("__underscores", case, "_underscores"));
    }

    #[test]
    fn validate_upper() {
        let case = Case::Upper;
        assert!(!is_valid("snake_case123", case, "SNAKE_CASE123"));
        assert!(!is_valid("snake_case_123", case, "SNAKE_CASE_123"));
        assert!(!is_valid("camelCase123", case, "CAMEL_CASE123"));
        assert!(!is_valid("PascalCase123", case, "PASCAL_CASE123"));
        assert!(!is_valid("ACRONYMCase123", case, "ACRONYM_CASE123"));
        assert!(is_valid("UPPER_CASE123", case, "UPPER_CASE123"));
        assert!(is_valid("_UNDERSCORE", case, "_UNDERSCORE"));
        assert!(!is_valid("__UNDERSCORES", case, "_UNDERSCORES"));
    }

    #[test]
    fn validate_pascal() {
        let case = Case::Pascal;
        assert!(!is_valid("snake_case123", case, "SnakeCase123"));
        assert!(!is_valid("snake_case_123", case, "SnakeCase123"));
        assert!(!is_valid("camelCase123", case, "CamelCase123"));
        assert!(is_valid("PascalCase123", case, "PascalCase123"));
        assert!(is_valid("ACRONYMCase123", case, "AcronymCase123"));
        assert!(!is_valid("UPPER_CASE123", case, "UpperCase123"));
        assert!(is_valid("_Underscore", case, "_Underscore"));
        assert!(!is_valid("__Underscores", case, "_Underscores"));
    }
}
