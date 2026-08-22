use itertools::Itertools;
use libtest_mimic::Failed;
use regex::Regex;
use std::str::FromStr;
use wgsl_parse::syntax::TranslationUnit;

const NUMBERED_IDENT_REGEX: &str = r"[v_]+([0-9]+)";

pub(crate) fn format(code: &str) -> Result<String, Failed> {
    let mut formatted_wgsl = TranslationUnit::from_str(code)?.to_string();
    let numbered_ident_regex = Regex::new(NUMBERED_IDENT_REGEX)?;
    let replaced_idents = numbered_ident_regex
        .captures_iter(&formatted_wgsl)
        .filter_map(|captures| {
            let ident = captures.get(0)?.as_str();
            let number = captures.get(1)?.as_str().parse::<u64>().ok()?;
            Some((ident, number))
        })
        .unique_by(|(ident, _)| *ident)
        .sorted_unstable_by_key(|(_, number)| *number)
        .enumerate()
        .map(|(index, (ident, _))| (ident.to_string(), format!("ident{index}")))
        .sorted_unstable_by_key(|(old_name, _)| usize::MAX - old_name.len()) // to avoid replacing variable prefixes
        .collect::<Vec<_>>();
    for (old_name, new_name) in &replaced_idents {
        formatted_wgsl = formatted_wgsl.replace(old_name, new_name);
    }
    Ok(formatted_wgsl)
}
