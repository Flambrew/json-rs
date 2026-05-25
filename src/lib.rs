//! This library parses JSON files
//!
//! # Example
//!
//! ```no_run
//! use flambrew_json_rs::*;
//! let path: &str = "example.json";
//! match parse_json(path) {
//!   Ok(res) => println!("Parsed JSON:\n{:#?}", res),
//!   Err(err) => match err {
//!     JErr::Io(msg) => println!("{}", msg),
//!     JErr::Parse(msg) => println!("{}", msg),
//!   },
//! }
//! ```

mod json;

/// JSON object name-value pair struct
///
/// Returned as part of parsed JSON
pub use json::NVPair;

/// JSON value struct
///
/// Returned as part of parsed JSON
pub use json::Value;

/// JSON error type
///
/// Returned when JSON parsing is not possible
pub use json::JErr;

/// JSON parser entrypoint
///
/// # Arguments
///
/// * 'path' - Path to JSON file
///
/// # Returns
///
/// - 'Ok(Value)' if parsing succeeds
/// - 'Err(Msg)' if the file cannot be read or is invalid JSON
///
/// # Example
///
/// ```no_run
/// use flambrew_json_rs::*;
/// let path: &str = "example.json";
/// let parsed_tree: Result<Value, JErr> = parse_json(path);
/// ```
pub use json::parse_json;
