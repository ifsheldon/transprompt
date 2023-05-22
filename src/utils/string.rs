use std::collections::{HashMap, HashSet};
use regex::{Captures, Regex};
use lazy_static::lazy_static;

lazy_static! {
    static ref PLACEHOLDER_MATCH_RE: Regex = Regex::new(r"\{\[.*?\]\}").unwrap();
}

#[inline]
fn strip_format(key: &str) -> &str {
    //! Strips "{\[" and "\]}" for a string, which is algorithmically unsafe.
    //! Ensure the string is properly formatted like "{\[a\]}".
    &key[2..key.len() - 2]
}

pub(crate) unsafe fn replace_all_placeholders(original: &str, mapping: &HashMap<String, Option<String>>) -> String {
    //! Replaces all placeholders with mappings.
    //!
    //! Safety:
    //! * Must ensure all keys in the string are in the mapping
    //! * Must ensure all entries in the mapping is Some
    //!
    let new_string = PLACEHOLDER_MATCH_RE.replace_all(original, |captures: &Captures| {
        let match_text = &captures[0];
        let key = strip_format(match_text);
        let replacement = mapping.get(key).unwrap_unchecked().as_ref().unwrap_unchecked();
        replacement
    });
    new_string.to_string()
}

pub fn get_placeholders(string: &str) -> HashSet<String> {
    PLACEHOLDER_MATCH_RE.captures_iter(string)
        .map(|captures| strip_format(&captures[0]).to_string())
        .collect()
}

#[cfg(test)]
mod string_tests {
    use std::collections::{HashMap, HashSet};
    use super::{get_placeholders, replace_all_placeholders};

    #[test]
    fn test_get_keys() {
        let string = "{[a]}";
        let keys = get_placeholders(string);
        let expect_keys = HashSet::from(["a".to_string()]);
        assert_eq!(expect_keys, keys);

        let string = "{[a\n]}";
        let keys = get_placeholders(string);
        assert_eq!(0, keys.len());

        let string = "{[a]}    {[b]}";
        let keys = get_placeholders(string);
        let expect_keys = HashSet::from(["a".to_string(), "b".to_string()]);
        assert_eq!(expect_keys, keys);
    }

    #[test]
    fn test_replace() {
        let string = "{[a]} and {[b]} and {[a]}";
        let keys = get_placeholders(string);
        let expect_keys = HashSet::from(["a".to_string(), "b".to_string()]);
        assert_eq!(expect_keys, keys);
        let mapping = HashMap::from([
            ("a".to_string(), Some("alice".to_string())),
            ("b".to_string(), Some("bob".to_string())),
        ]);

        assert_eq!("alice and bob and alice", unsafe { replace_all_placeholders(string, &mapping).as_str() });
    }
}