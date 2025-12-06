//! String manipulation functions.
//!
//! This module provides extended string operations beyond the standard JMESPath built-ins.
//!
//! # Function Reference
//!
//! | Function | Signature | Description |
//! |----------|-----------|-------------|
//! | [`lower`](#lower) | `lower(string) → string` | Convert to lowercase |
//! | [`upper`](#upper) | `upper(string) → string` | Convert to uppercase |
//! | [`trim`](#trim) | `trim(string) → string` | Remove leading/trailing whitespace |
//! | [`trim_left`](#trim_left) | `trim_left(string) → string` | Remove leading whitespace |
//! | [`trim_right`](#trim_right) | `trim_right(string) → string` | Remove trailing whitespace |
//! | [`split`](#split) | `split(string, delimiter) → array` | Split string into array |
//! | [`replace`](#replace) | `replace(string, old, new) → string` | Replace all occurrences |
//! | [`pad_left`](#pad_left) | `pad_left(string, width, char) → string` | Left-pad to width |
//! | [`pad_right`](#pad_right) | `pad_right(string, width, char) → string` | Right-pad to width |
//! | [`substr`](#substr) | `substr(string, start, length?) → string` | Extract substring |
//! | [`capitalize`](#capitalize) | `capitalize(string) → string` | Capitalize first letter |
//! | [`title`](#title) | `title(string) → string` | Title Case Each Word |
//! | [`repeat`](#repeat) | `repeat(string, count) → string` | Repeat string N times |
//! | [`find_first`](#find_first) | `find_first(string, search) → number` | Find first occurrence |
//! | [`find_last`](#find_last) | `find_last(string, search) → number` | Find last occurrence |
//! | [`slice`](#slice) | `slice(string, start, end?) → string` | Extract slice |
//! | [`concat`](#concat) | `concat(array, separator?) → string` | Join array of strings |
//! | [`camel_case`](#camel_case) | `camel_case(string) → string` | Convert to camelCase |
//! | [`snake_case`](#snake_case) | `snake_case(string) → string` | Convert to snake_case |
//! | [`kebab_case`](#kebab_case) | `kebab_case(string) → string` | Convert to kebab-case |
//! | [`truncate`](#truncate) | `truncate(string, length, suffix?) → string` | Truncate with suffix |
//! | [`wrap`](#wrap) | `wrap(string, width) → string` | Word-wrap to width |
//! | [`format`](#format) | `format(template, ...args) → string` | Format with placeholders |
//!
//! # Examples
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::string;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! string::register(&mut runtime);
//!
//! // Case conversion
//! let expr = runtime.compile("upper(@)").unwrap();
//! let result = expr.search(&Variable::String("hello".into())).unwrap();
//! assert_eq!(result.as_string().unwrap(), "HELLO");
//!
//! // Split and join
//! let expr = runtime.compile("split(@, ',')").unwrap();
//! let result = expr.search(&Variable::String("a,b,c".into())).unwrap();
//! assert_eq!(result.as_array().unwrap().len(), 3);
//! ```
//!
//! # Function Details
//!
//! ## lower
//!
//! Converts a string to lowercase.
//!
//! ```text
//! lower(string) → string
//!
//! lower('HELLO')      → "hello"
//! lower('Hello World') → "hello world"
//! lower('123ABC')     → "123abc"
//! ```
//!
//! ## upper
//!
//! Converts a string to uppercase.
//!
//! ```text
//! upper(string) → string
//!
//! upper('hello')      → "HELLO"
//! upper('Hello World') → "HELLO WORLD"
//! upper('abc123')     → "ABC123"
//! ```
//!
//! ## trim
//!
//! Removes leading and trailing whitespace from a string.
//!
//! ```text
//! trim(string) → string
//!
//! trim('  hello  ')   → "hello"
//! trim('\t\nhello\n') → "hello"
//! trim('hello')       → "hello"
//! ```
//!
//! ## trim_left
//!
//! Removes leading whitespace from a string.
//!
//! ```text
//! trim_left(string) → string
//!
//! trim_left('  hello  ') → "hello  "
//! trim_left('\nhello')   → "hello"
//! ```
//!
//! ## trim_right
//!
//! Removes trailing whitespace from a string.
//!
//! ```text
//! trim_right(string) → string
//!
//! trim_right('  hello  ') → "  hello"
//! trim_right('hello\n')   → "hello"
//! ```
//!
//! ## split
//!
//! Splits a string into an array using a delimiter.
//!
//! ```text
//! split(string, delimiter) → array
//!
//! split('a,b,c', ',')       → ["a", "b", "c"]
//! split('hello world', ' ') → ["hello", "world"]
//! split('a::b::c', '::')    → ["a", "b", "c"]
//! split('hello', ',')       → ["hello"]
//! ```
//!
//! ## replace
//!
//! Replaces all occurrences of a substring with another string.
//!
//! ```text
//! replace(string, old, new) → string
//!
//! replace('hello world', 'world', 'rust')  → "hello rust"
//! replace('aaa', 'a', 'b')                 → "bbb"
//! replace('hello', 'x', 'y')               → "hello"
//! ```
//!
//! ## pad_left
//!
//! Left-pads a string to a specified width with a character.
//!
//! ```text
//! pad_left(string, width, char) → string
//!
//! pad_left('42', 5, '0')    → "00042"
//! pad_left('hello', 10, ' ') → "     hello"
//! pad_left('hello', 3, 'x')  → "hello"  // No padding if already >= width
//! ```
//!
//! ## pad_right
//!
//! Right-pads a string to a specified width with a character.
//!
//! ```text
//! pad_right(string, width, char) → string
//!
//! pad_right('42', 5, '0')    → "42000"
//! pad_right('hello', 10, '.') → "hello....."
//! pad_right('hello', 3, 'x')  → "hello"  // No padding if already >= width
//! ```
//!
//! ## substr
//!
//! Extracts a substring starting at an index with optional length.
//! Supports negative indices (from end of string).
//!
//! ```text
//! substr(string, start, length?) → string
//!
//! substr('hello', 0, 2)   → "he"
//! substr('hello', 2)      → "llo"
//! substr('hello', -2)     → "lo"       // Last 2 characters
//! substr('hello', -3, 2)  → "ll"       // 2 chars starting 3 from end
//! ```
//!
//! ## capitalize
//!
//! Capitalizes the first letter of a string.
//!
//! ```text
//! capitalize(string) → string
//!
//! capitalize('hello')       → "Hello"
//! capitalize('HELLO')       → "HELLO"
//! capitalize('hello world') → "Hello world"
//! ```
//!
//! ## title
//!
//! Converts a string to title case (capitalizes each word).
//!
//! ```text
//! title(string) → string
//!
//! title('hello world')      → "Hello World"
//! title('the quick brown fox') → "The Quick Brown Fox"
//! title('HELLO WORLD')      → "Hello World"
//! ```
//!
//! ## repeat
//!
//! Repeats a string a specified number of times.
//!
//! ```text
//! repeat(string, count) → string
//!
//! repeat('ab', 3)   → "ababab"
//! repeat('-', 10)   → "----------"
//! repeat('hello', 0) → ""
//! ```
//!
//! ## find_first
//!
//! Finds the first occurrence of a substring. Returns -1 if not found.
//!
//! ```text
//! find_first(string, search) → number
//!
//! find_first('hello world', 'world') → 6
//! find_first('hello world', 'o')     → 4
//! find_first('hello', 'x')           → -1
//! ```
//!
//! ## find_last
//!
//! Finds the last occurrence of a substring. Returns -1 if not found.
//!
//! ```text
//! find_last(string, search) → number
//!
//! find_last('hello world', 'o') → 7
//! find_last('abcabc', 'abc')    → 3
//! find_last('hello', 'x')       → -1
//! ```
//!
//! ## slice
//!
//! Extracts a portion of a string. Supports negative indices.
//!
//! ```text
//! slice(string, start, end?) → string
//!
//! slice('hello', 1, 4)   → "ell"
//! slice('hello', 2)      → "llo"
//! slice('hello', -3)     → "llo"      // From 3rd-to-last to end
//! slice('hello', 0, -1)  → "hell"     // From start to 1 before end
//! slice('hello', -3, -1) → "ll"       // From 3rd-to-last to 1 before end
//! ```
//!
//! ## concat
//!
//! Joins an array of strings with an optional separator.
//!
//! ```text
//! concat(array, separator?) → string
//!
//! concat(['a', 'b', 'c'], ',')  → "a,b,c"
//! concat(['a', 'b', 'c'])       → "abc"
//! concat(['hello', 'world'], ' ') → "hello world"
//! ```
//!
//! ## camel_case
//!
//! Converts a string to camelCase.
//!
//! ```text
//! camel_case(string) → string
//!
//! camel_case('hello_world')   → "helloWorld"
//! camel_case('hello-world')   → "helloWorld"
//! camel_case('Hello World')   → "helloWorld"
//! camel_case('XMLHttpRequest') → "xmlhttprequest"
//! ```
//!
//! ## snake_case
//!
//! Converts a string to snake_case.
//!
//! ```text
//! snake_case(string) → string
//!
//! snake_case('helloWorld')    → "hello_world"
//! snake_case('HelloWorld')    → "hello_world"
//! snake_case('hello-world')   → "hello_world"
//! snake_case('hello world')   → "hello_world"
//! ```
//!
//! ## kebab_case
//!
//! Converts a string to kebab-case.
//!
//! ```text
//! kebab_case(string) → string
//!
//! kebab_case('helloWorld')    → "hello-world"
//! kebab_case('HelloWorld')    → "hello-world"
//! kebab_case('hello_world')   → "hello-world"
//! kebab_case('hello world')   → "hello-world"
//! ```
//!
//! ## truncate
//!
//! Truncates a string to a maximum length, adding a suffix if truncated.
//! Default suffix is "...".
//!
//! ```text
//! truncate(string, length, suffix?) → string
//!
//! truncate('hello world', 8)        → "hello..."
//! truncate('hello world', 8, '…')   → "hello w…"
//! truncate('hello', 10)             → "hello"      // No truncation needed
//! truncate('hello world', 5, '')    → "hello"      // No suffix
//! ```
//!
//! ## wrap
//!
//! Word-wraps a string to a specified width, returning a string with newlines.
//!
//! ```text
//! wrap(string, width) → string
//!
//! wrap('hello world', 5)            → "hello\nworld"
//! wrap('the quick brown fox', 10)   → "the quick\nbrown fox"
//! wrap('hello', 100)                → "hello"
//! ```
//!
//! ## format
//!
//! Formats a string by replacing `{0}`, `{1}`, etc. with arguments.
//!
//! ```text
//! format(template, ...args) → string
//!
//! format('Hello, {0}!', 'World')           → "Hello, World!"
//! format('{0} + {1} = {2}', 1, 2, 3)       → "1 + 2 = 3"
//! format('Name: {0}, Age: {1}', 'Alice', 30) → "Name: Alice, Age: 30"
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all string functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("lower", Box::new(LowerFn::new()));
    runtime.register_function("upper", Box::new(UpperFn::new()));
    runtime.register_function("trim", Box::new(TrimFn::new()));
    runtime.register_function("trim_left", Box::new(TrimStartFn::new()));
    runtime.register_function("trim_right", Box::new(TrimEndFn::new()));
    runtime.register_function("split", Box::new(SplitFn::new()));
    runtime.register_function("replace", Box::new(ReplaceFn::new()));
    runtime.register_function("pad_left", Box::new(PadLeftFn::new()));
    runtime.register_function("pad_right", Box::new(PadRightFn::new()));
    runtime.register_function("substr", Box::new(SubstrFn::new()));
    runtime.register_function("capitalize", Box::new(CapitalizeFn::new()));
    runtime.register_function("title", Box::new(TitleFn::new()));
    runtime.register_function("repeat", Box::new(RepeatFn::new()));
    runtime.register_function("find_first", Box::new(IndexOfFn::new()));
    runtime.register_function("find_last", Box::new(LastIndexOfFn::new()));
    runtime.register_function("slice", Box::new(SliceFn::new()));
    runtime.register_function("concat", Box::new(ConcatFn::new()));
    runtime.register_function("upper_case", Box::new(UpperCaseFn::new()));
    runtime.register_function("lower_case", Box::new(LowerCaseFn::new()));
    runtime.register_function("title_case", Box::new(TitleCaseFn::new()));
    runtime.register_function("camel_case", Box::new(CamelCaseFn::new()));
    runtime.register_function("snake_case", Box::new(SnakeCaseFn::new()));
    runtime.register_function("kebab_case", Box::new(KebabCaseFn::new()));
    runtime.register_function("truncate", Box::new(TruncateFn::new()));
    runtime.register_function("wrap", Box::new(WrapFn::new()));
    runtime.register_function("format", Box::new(FormatFn::new()));
}

// =============================================================================
// lower(string) -> string
// =============================================================================

define_function!(LowerFn, vec![ArgumentType::String], None);

impl Function for LowerFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.to_lowercase())))
    }
}

// =============================================================================
// upper(string) -> string
// =============================================================================

define_function!(UpperFn, vec![ArgumentType::String], None);

impl Function for UpperFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.to_uppercase())))
    }
}

// =============================================================================
// trim(string) -> string
// =============================================================================

define_function!(TrimFn, vec![ArgumentType::String], None);

impl Function for TrimFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.trim().to_string())))
    }
}

// =============================================================================
// trim_start(string) -> string
// =============================================================================

define_function!(TrimStartFn, vec![ArgumentType::String], None);

impl Function for TrimStartFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.trim_start().to_string())))
    }
}

// =============================================================================
// trim_end(string) -> string
// =============================================================================

define_function!(TrimEndFn, vec![ArgumentType::String], None);

impl Function for TrimEndFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.trim_end().to_string())))
    }
}

// =============================================================================
// split(string, delimiter) -> array
// =============================================================================

define_function!(
    SplitFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for SplitFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let delimiter = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string delimiter".to_owned()),
            )
        })?;

        let parts: Vec<Rcvar> = s
            .split(delimiter)
            .map(|part| Rc::new(Variable::String(part.to_string())) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(parts)))
    }
}

// =============================================================================
// replace(string, old, new) -> string
// =============================================================================

define_function!(
    ReplaceFn,
    vec![
        ArgumentType::String,
        ArgumentType::String,
        ArgumentType::String
    ],
    None
);

impl Function for ReplaceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let old = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected old string argument".to_owned()),
            )
        })?;

        let new = args[2].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected new string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.replace(old, new))))
    }
}

// =============================================================================
// pad_left(string, width, char) -> string
// =============================================================================

define_function!(
    PadLeftFn,
    vec![
        ArgumentType::String,
        ArgumentType::Number,
        ArgumentType::String
    ],
    None
);

impl Function for PadLeftFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let width = args[1].as_number().map(|n| n as usize).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected positive number for width".to_owned()),
            )
        })?;

        let pad_char = args[2].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for pad character".to_owned()),
            )
        })?;

        let pad = pad_char.chars().next().unwrap_or(' ');
        let result = if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", pad.to_string().repeat(width - s.len()), s)
        };

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// pad_right(string, width, char) -> string
// =============================================================================

define_function!(
    PadRightFn,
    vec![
        ArgumentType::String,
        ArgumentType::Number,
        ArgumentType::String
    ],
    None
);

impl Function for PadRightFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let width = args[1].as_number().map(|n| n as usize).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected positive number for width".to_owned()),
            )
        })?;

        let pad_char = args[2].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for pad character".to_owned()),
            )
        })?;

        let pad = pad_char.chars().next().unwrap_or(' ');
        let result = if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", s, pad.to_string().repeat(width - s.len()))
        };

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// substr(string, start, length?) -> string
// =============================================================================

define_function!(
    SubstrFn,
    vec![ArgumentType::String, ArgumentType::Number],
    Some(ArgumentType::Number)
);

impl Function for SubstrFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let start = args[1].as_number().map(|n| n as i64).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for start".to_owned()),
            )
        })?;

        // Handle negative start (from end)
        let start_idx = if start < 0 {
            (s.len() as i64 + start).max(0) as usize
        } else {
            start as usize
        };

        let result = if args.len() > 2 {
            let length = args[2].as_number().map(|n| n as usize).ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected positive number for length".to_owned()),
                )
            })?;
            s.chars().skip(start_idx).take(length).collect()
        } else {
            s.chars().skip(start_idx).collect()
        };

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// capitalize(string) -> string (first letter uppercase)
// =============================================================================

define_function!(CapitalizeFn, vec![ArgumentType::String], None);

impl Function for CapitalizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let result = if s.is_empty() {
            String::new()
        } else {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        };

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// title(string) -> string (capitalize each word)
// =============================================================================

define_function!(TitleFn, vec![ArgumentType::String], None);

impl Function for TitleFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let result = s
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().to_string() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// repeat(string, count) -> string
// =============================================================================

define_function!(
    RepeatFn,
    vec![ArgumentType::String, ArgumentType::Number],
    None
);

impl Function for RepeatFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let count = args[1].as_number().map(|n| n as usize).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected positive number for count".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.repeat(count))))
    }
}

// =============================================================================
// index_of(string, search) -> number (-1 if not found)
// =============================================================================

define_function!(
    IndexOfFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for IndexOfFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let search = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected search string".to_owned()),
            )
        })?;

        let result = s.find(search).map(|i| i as i64).unwrap_or(-1);

        Ok(Rc::new(Variable::Number(serde_json::Number::from(result))))
    }
}

// =============================================================================
// last_index_of(string, search) -> number (-1 if not found)
// =============================================================================

define_function!(
    LastIndexOfFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for LastIndexOfFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let search = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected search string".to_owned()),
            )
        })?;

        let result = s.rfind(search).map(|i| i as i64).unwrap_or(-1);

        Ok(Rc::new(Variable::Number(serde_json::Number::from(result))))
    }
}

// =============================================================================
// slice(string, start, end?) -> string
// =============================================================================

define_function!(
    SliceFn,
    vec![ArgumentType::String, ArgumentType::Number],
    Some(ArgumentType::Number)
);

impl Function for SliceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let len = s.len() as i64;

        let start = args[1].as_number().map(|n| n as i64).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for start".to_owned()),
            )
        })?;

        // Handle negative indices
        let start_idx = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len) as usize
        };

        let end_idx = if args.len() > 2 {
            let end = args[2].as_number().map(|n| n as i64).ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for end".to_owned()),
                )
            })?;
            if end < 0 {
                (len + end).max(0) as usize
            } else {
                end.min(len) as usize
            }
        } else {
            len as usize
        };

        let result: String = s
            .chars()
            .skip(start_idx)
            .take(end_idx.saturating_sub(start_idx))
            .collect();

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// concat(array_of_strings, separator?) -> string
// =============================================================================

define_function!(
    ConcatFn,
    vec![ArgumentType::Array],
    Some(ArgumentType::String)
);

impl Function for ConcatFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let separator = if args.len() > 1 {
            args[1]
                .as_string()
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let strings: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_string().map(|s| s.to_string()))
            .collect();

        Ok(Rc::new(Variable::String(strings.join(&separator))))
    }
}

// =============================================================================
// upper_case(string) -> string (alias for upper, snake_case style)
// =============================================================================

define_function!(UpperCaseFn, vec![ArgumentType::String], None);

impl Function for UpperCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.to_uppercase())))
    }
}

// =============================================================================
// lower_case(string) -> string (alias for lower, snake_case style)
// =============================================================================

define_function!(LowerCaseFn, vec![ArgumentType::String], None);

impl Function for LowerCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(s.to_lowercase())))
    }
}

// =============================================================================
// title_case(string) -> string (alias for title, snake_case style)
// =============================================================================

define_function!(TitleCaseFn, vec![ArgumentType::String], None);

impl Function for TitleCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let result = s
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().to_string() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// camel_case(string) -> string (helloWorld)
// =============================================================================

define_function!(CamelCaseFn, vec![ArgumentType::String], None);

impl Function for CamelCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut result = String::new();
        let mut capitalize_next = false;
        let mut first_word = true;

        for c in s.chars() {
            if c.is_alphanumeric() {
                if capitalize_next && !first_word {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c.to_ascii_lowercase());
                }
                first_word = false;
            } else {
                capitalize_next = true;
            }
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// snake_case(string) -> string (hello_world)
// =============================================================================

define_function!(SnakeCaseFn, vec![ArgumentType::String], None);

impl Function for SnakeCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut result = String::new();
        let mut prev_was_lower = false;

        for c in s.chars() {
            if c.is_uppercase() {
                if prev_was_lower && !result.is_empty() {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_was_lower = false;
            } else if c.is_alphanumeric() {
                result.push(c.to_ascii_lowercase());
                prev_was_lower = c.is_lowercase();
            } else if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
                prev_was_lower = false;
            }
        }

        // Trim trailing underscore
        if result.ends_with('_') {
            result.pop();
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// kebab_case(string) -> string (hello-world)
// =============================================================================

define_function!(KebabCaseFn, vec![ArgumentType::String], None);

impl Function for KebabCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut result = String::new();
        let mut prev_was_lower = false;

        for c in s.chars() {
            if c.is_uppercase() {
                if prev_was_lower && !result.is_empty() {
                    result.push('-');
                }
                result.push(c.to_ascii_lowercase());
                prev_was_lower = false;
            } else if c.is_alphanumeric() {
                result.push(c.to_ascii_lowercase());
                prev_was_lower = c.is_lowercase();
            } else if !result.is_empty() && !result.ends_with('-') {
                result.push('-');
                prev_was_lower = false;
            }
        }

        // Trim trailing hyphen
        if result.ends_with('-') {
            result.pop();
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// truncate(string, length, suffix?) -> string
// =============================================================================

define_function!(
    TruncateFn,
    vec![ArgumentType::String, ArgumentType::Number],
    Some(ArgumentType::String)
);

impl Function for TruncateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let max_len = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for length".to_owned()),
            )
        })? as usize;

        let suffix = args
            .get(2)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "...".to_string());

        if s.len() <= max_len {
            Ok(Rc::new(Variable::String(s.to_string())))
        } else {
            let truncate_at = max_len.saturating_sub(suffix.len());
            let truncated: String = s.chars().take(truncate_at).collect();
            Ok(Rc::new(Variable::String(format!(
                "{}{}",
                truncated, suffix
            ))))
        }
    }
}

// =============================================================================
// wrap(string, width) -> string with newlines
// =============================================================================

define_function!(
    WrapFn,
    vec![ArgumentType::String, ArgumentType::Number],
    None
);

impl Function for WrapFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let width = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for width".to_owned()),
            )
        })? as usize;

        if width == 0 {
            return Ok(Rc::new(Variable::String(s.to_string())));
        }

        let mut lines: Vec<String> = Vec::new();

        // Process each paragraph (separated by newlines) separately
        for paragraph in s.split('\n') {
            let mut current_line = String::new();

            for word in paragraph.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }

            // Push the last line of this paragraph (even if empty to preserve blank lines)
            lines.push(current_line);
        }

        // Remove trailing empty line if the input didn't end with a newline
        if !s.ends_with('\n') && lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }

        // If we have no lines but had input, return the original
        if lines.is_empty() && !s.is_empty() {
            return Ok(Rc::new(Variable::String(s.to_string())));
        }

        // Join lines with newlines and return as string
        Ok(Rc::new(Variable::String(lines.join("\n"))))
    }
}

// =============================================================================
// format(template, ...args) -> string
// =============================================================================

define_function!(
    FormatFn,
    vec![ArgumentType::String],
    Some(ArgumentType::Any)
);

impl Function for FormatFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let template = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected template string".to_owned()),
            )
        })?;

        let mut result = template.to_string();

        for (i, arg) in args.iter().skip(1).enumerate() {
            let placeholder = format!("{{{}}}", i);
            let value = match &**arg {
                Variable::String(s) => s.clone(),
                Variable::Number(n) => n.to_string(),
                Variable::Bool(b) => b.to_string(),
                Variable::Null => "null".to_string(),
                _ => serde_json::to_string(&**arg).unwrap_or_else(|_| "null".to_string()),
            };
            result = result.replace(&placeholder, &value);
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jmespath::Runtime;

    fn setup_runtime() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        runtime
    }

    #[test]
    fn test_lower() {
        let runtime = setup_runtime();
        let expr = runtime.compile("lower(@)").unwrap();
        let data = Variable::String("HELLO".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_upper() {
        let runtime = setup_runtime();
        let expr = runtime.compile("upper(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "HELLO");
    }

    #[test]
    fn test_trim() {
        let runtime = setup_runtime();
        let expr = runtime.compile("trim(@)").unwrap();
        let data = Variable::String("  hello  ".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_split() {
        let runtime = setup_runtime();
        let expr = runtime.compile("split(@, ',')").unwrap();
        let data = Variable::String("a,b,c".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "a");
    }

    #[test]
    fn test_camel_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("camel_case(@)").unwrap();
        let data = Variable::String("hello_world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "helloWorld");
    }

    #[test]
    fn test_snake_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("snake_case(@)").unwrap();
        let data = Variable::String("helloWorld".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello_world");
    }

    #[test]
    fn test_wrap_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("wrap(@, `5`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello\nworld");
    }

    #[test]
    fn test_wrap_preserves_newlines() {
        let runtime = setup_runtime();
        let expr = runtime.compile("wrap(@, `100`)").unwrap();
        let data = Variable::String("hello\nworld".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello\nworld");
    }

    #[test]
    fn test_wrap_wide_width() {
        let runtime = setup_runtime();
        let expr = runtime.compile("wrap(@, `100`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }
}
