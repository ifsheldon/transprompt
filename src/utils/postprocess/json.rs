use serde_json::{from_str, Value};
use anyhow::Result;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;


/// Filters invalid content and tries to parse the valid json string.
///
/// Returns a [serde_json::Value] if the string is valid json else an [InvalidJSON] error.
///
/// # Example
/// ```
/// use transprompt::utils::postprocess::json::filter_to_json;
/// let valid_str = "partially valid: {\"a\":\"alice\"}";
/// let json_value = filter_to_json(valid_str).expect("Expect to be fine but failed");
/// assert_eq!(json_value["a"], "alice");
///
/// let invalid_str = "partially valid: \"a\":\"alice\"}";
/// assert_eq!(filter_to_json(invalid_str).is_err(), true)
/// ```
pub fn filter_to_json(string: impl Into<String>) -> Result<Value> {
    let string = string.into();
    let left_brace_idx = string.find("{");
    let right_brace_idx = string.rfind("}");
    match (left_brace_idx, right_brace_idx) {
        (Some(lbi), Some(rbi)) => {
            let valid_json = &string[lbi..rbi + 1];
            let value = from_str(valid_json)?;
            Ok(value)
        }
        _ => Err(InvalidJSON { invalid_string: string }.into())
    }
}

/// Use Jsonformer to get a JSON from a string.
pub struct Jsonformer {
    // TODO: add Jsonformer support
}

/// Error when the string is not a valid json.
#[derive(Debug, Clone)]
pub struct InvalidJSON {
    pub invalid_string: String,
}


impl fmt::Display for InvalidJSON {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid string to be parsed:\n{}", self.invalid_string)
    }
}

impl Error for InvalidJSON {}


#[cfg(test)]
mod test_json {
    use crate::utils::postprocess::json::filter_to_json;

    #[test]
    fn test_filter_to_json() {
        let valid_str = "{\"a\":\"alice\"}";
        let json_value = filter_to_json(valid_str).expect("Expect to be fine but failed");
        println!("{:?}", json_value);

        let valid_str = "Here is the result you ask for: {\"a\":\"alice\"}";
        let json_value = filter_to_json(valid_str).expect("Expect to be fine but failed");
        println!("{:?}", json_value);

        let invalid_str = "Here is the result you ask for: {\"a\":\"alice\"";
        let json_error = filter_to_json(invalid_str).expect_err("This should give error but not");
        println!("{:?}", json_error);

        let invalid_str = "{{}}";
        let json_error = filter_to_json(invalid_str).expect_err("This should give error but not");
        println!("{:?}", json_error);
    }
}