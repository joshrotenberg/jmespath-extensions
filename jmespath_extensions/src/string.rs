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
//! | [`find_first`](#find_first) | `find_first(string, search, start?, end?) → number\|null` | Find first occurrence (JEP-014) |
//! | [`find_last`](#find_last) | `find_last(string, search, start?, end?) → number\|null` | Find last occurrence (JEP-014) |
//! | [`slice`](#slice) | `slice(string, start, end?) → string` | Extract slice |
//! | [`concat`](#concat) | `concat(array, separator?) → string` | Join array of strings |
//! | [`camel_case`](#camel_case) | `camel_case(string) → string` | Convert to camelCase |
//! | [`snake_case`](#snake_case) | `snake_case(string) → string` | Convert to snake_case |
//! | [`kebab_case`](#kebab_case) | `kebab_case(string) → string` | Convert to kebab-case |
//! | [`truncate`](#truncate) | `truncate(string, length, suffix?) → string` | Truncate with suffix |
//! | [`wrap`](#wrap) | `wrap(string, width) → string` | Word-wrap to width |
//! | [`format`](#format) | `format(template, args) → string` | Format with placeholders |
//! | [`sprintf`](#sprintf) | `sprintf(format, ...args) → string` | Printf-style formatting |
//! | [`ltrimstr`](#ltrimstr) | `ltrimstr(string, prefix) → string` | Remove prefix if present |
//! | [`rtrimstr`](#rtrimstr) | `rtrimstr(string, suffix) → string` | Remove suffix if present |
//! | [`indices`](#indices) | `indices(string, search) → array` | Find all occurrence indices |
//! | [`inside`](#inside) | `inside(string, search) → boolean` | Check if search is contained |
//! | [`humanize`](#humanize) | `humanize(string) → string` | Convert to human-readable form |
//! | [`deburr`](#deburr) | `deburr(string) → string` | Remove diacritical marks |
//! | [`words`](#words) | `words(string) → array` | Split into array of words |
//! | [`escape`](#escape) | `escape(string) → string` | Escape HTML entities |
//! | [`unescape`](#unescape) | `unescape(string) → string` | Unescape HTML entities |
//! | [`escape_regex`](#escape_regex) | `escape_regex(string) → string` | Escape regex special chars |
//! | [`start_case`](#start_case) | `start_case(string) → string` | Convert To Start Case |
//! | [`mask`](#mask) | `mask(string, visible?, char?) → string` | Mask string, keep last N visible |
//! | [`redact`](#redact) | `redact(string, pattern, replacement?) → string` | Redact regex matches |
//! | [`normalize_whitespace`](#normalize_whitespace) | `normalize_whitespace(string) → string` | Collapse whitespace |
//! | [`is_blank`](#is_blank) | `is_blank(string) → boolean` | Check if empty/whitespace |
//! | [`abbreviate`](#abbreviate) | `abbreviate(string, max, suffix?) → string` | Truncate with ellipsis |
//! | [`center`](#center) | `center(string, width, char?) → string` | Center-pad string |
//! | [`reverse_string`](#reverse_string) | `reverse_string(string) → string` | Reverse string |
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
//! Finds the first occurrence of a substring. Returns `null` if not found.
//! Implements [JEP-014](https://github.com/jmespath-community/jmespath.spec/blob/main/jep-014-string-functions.md).
//!
//! Optional `start` and `end` parameters restrict the search to a slice of the string.
//! Negative indices count from the end of the string.
//!
//! ```text
//! find_first(string, search, start?, end?) → number | null
//!
//! find_first('hello world', 'world')       → 6
//! find_first('hello world', 'o')           → 4
//! find_first('hello', 'x')                 → null
//! find_first('hello world', 'o', 5)        → 7      // Search from index 5
//! find_first('hello world', 'o', 0, 5)     → 4      // Search in range [0, 5)
//! find_first('hello world', 'o', -5)       → 7      // Search from 5th-to-last char
//! ```
//!
//! ## find_last
//!
//! Finds the last occurrence of a substring. Returns `null` if not found.
//! Implements [JEP-014](https://github.com/jmespath-community/jmespath.spec/blob/main/jep-014-string-functions.md).
//!
//! Optional `start` and `end` parameters restrict the search to a slice of the string.
//! Negative indices count from the end of the string.
//!
//! ```text
//! find_last(string, search, start?, end?) → number | null
//!
//! find_last('hello world', 'o')            → 7
//! find_last('abcabc', 'abc')               → 3
//! find_last('hello', 'x')                  → null
//! find_last('hello world', 'o', 0, 6)      → 4      // Search in range [0, 6)
//! find_last('hello world', 'l', 0, -1)     → 9      // Exclude last char
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
//! Formats a string using positional or named placeholders.
//! Supports three modes:
//! - Positional with array: `format('Hello {0}', ['World'])`
//! - Named with object: `format('Hello {name}', {name: 'World'})`
//! - Variadic: `format('Hello {0}', 'World')`
//!
//! ```text
//! format(template, args) → string
//!
//! # Positional with array
//! format('Hello {0}, you have {1} messages', ['Alice', 5])
//! → "Hello Alice, you have 5 messages"
//!
//! # Named with object
//! format('Hello {name}!', {name: 'World'})
//! → "Hello World!"
//!
//! # Variadic
//! format('Name: {0}, Age: {1}', 'Alice', 30) → "Name: Alice, Age: 30"
//! ```
//!
//! ## sprintf
//!
//! Printf-style formatting with format specifiers.
//!
//! ```text
//! sprintf(format, ...args) → string
//!
//! Supported format specifiers:
//!   %s  - String
//!   %d  - Integer (decimal)
//!   %i  - Integer (decimal)
//!   %f  - Floating point (default 6 decimal places)
//!   %e  - Scientific notation
//!   %x  - Hexadecimal (lowercase)
//!   %X  - Hexadecimal (uppercase)
//!   %o  - Octal
//!   %b  - Binary
//!   %c  - Character (from number or first char of string)
//!   %%  - Literal percent sign
//!
//! Width and precision:
//!   %10s   - Right-pad to 10 characters
//!   %-10s  - Left-pad to 10 characters
//!   %.2f   - 2 decimal places
//!   %8.2f  - 8 wide, 2 decimal places
//!
//! sprintf('Hello, %s!', 'World')        → "Hello, World!"
//! sprintf('%d + %d = %d', 1, 2, 3)     → "1 + 2 = 3"
//! sprintf('Pi is %.2f', 3.14159)       → "Pi is 3.14"
//! sprintf('Hex: %x, Bin: %b', 255, 10) → "Hex: ff, Bin: 1010"
//! sprintf('%10s', 'hi')                → "        hi"
//! sprintf('100%% done')                → "100% done"
//! ```
//!
//! ## ltrimstr
//!
//! Removes a prefix from a string if it starts with that prefix.
//!
//! ```text
//! ltrimstr(string, prefix) → string
//!
//! ltrimstr('hello world', 'hello ')  → "world"
//! ltrimstr('foobar', 'foo')          → "bar"
//! ltrimstr('foobar', 'bar')          → "foobar"  // No change, doesn't start with 'bar'
//! ltrimstr('', 'foo')                → ""
//! ```
//!
//! ## rtrimstr
//!
//! Removes a suffix from a string if it ends with that suffix.
//!
//! ```text
//! rtrimstr(string, suffix) → string
//!
//! rtrimstr('hello world', ' world')  → "hello"
//! rtrimstr('foobar', 'bar')          → "foo"
//! rtrimstr('foobar', 'foo')          → "foobar"  // No change, doesn't end with 'foo'
//! rtrimstr('', 'foo')                → ""
//! ```
//!
//! ## indices
//!
//! Finds all indices where a substring occurs in a string.
//! Returns an array of indices (0-based).
//!
//! ```text
//! indices(string, search) → array
//!
//! indices('abcabc', 'bc')      → [1, 4]
//! indices('hello', 'l')        → [2, 3]
//! indices('hello', 'x')        → []
//! indices('aaa', 'aa')         → [0, 1]  // Overlapping matches
//! ```
//!
//! ## inside
//!
//! Checks if a string contains another string.
//! This is the inverse of `contains` - useful when the search string is the subject.
//!
//! ```text
//! inside(search, string) → boolean
//!
//! inside('world', 'hello world')  → true
//! inside('foo', 'hello world')    → false
//! inside('', 'hello')             → true   // Empty string is in any string
//! inside('hello', '')             → false  // Non-empty can't be in empty
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
    runtime.register_function("sprintf", Box::new(SprintfFn::new()));
    runtime.register_function("ltrimstr", Box::new(LtrimstrFn::new()));
    runtime.register_function("rtrimstr", Box::new(RtrimstrFn::new()));
    runtime.register_function("indices", Box::new(IndicesFn::new()));
    runtime.register_function("inside", Box::new(InsideFn::new()));
    runtime.register_function("humanize", Box::new(HumanizeFn::new()));
    runtime.register_function("deburr", Box::new(DeburrrFn::new()));
    runtime.register_function("words", Box::new(WordsFn::new()));
    runtime.register_function("escape", Box::new(EscapeFn::new()));
    runtime.register_function("unescape", Box::new(UnescapeFn::new()));
    runtime.register_function("escape_regex", Box::new(EscapeRegexFn::new()));
    runtime.register_function("start_case", Box::new(StartCaseFn::new()));
    runtime.register_function("mask", Box::new(MaskFn::new()));
    #[cfg(feature = "regex")]
    runtime.register_function("redact", Box::new(RedactFn::new()));
    runtime.register_function(
        "normalize_whitespace",
        Box::new(NormalizeWhitespaceFn::new()),
    );
    runtime.register_function("is_blank", Box::new(IsBlankFn::new()));
    runtime.register_function("abbreviate", Box::new(AbbreviateFn::new()));
    runtime.register_function("center", Box::new(CenterFn::new()));
    runtime.register_function("reverse_string", Box::new(ReverseStringFn::new()));
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
// index_of(string, search, start?, end?) -> number | null (JEP-014)
// =============================================================================

define_function!(
    IndexOfFn,
    vec![ArgumentType::String, ArgumentType::String],
    Some(ArgumentType::Number)
);

/// Helper to normalize an index (handling negative values) within a string length
fn normalize_index(idx: i64, len: usize) -> usize {
    if idx < 0 {
        (len as i64 + idx).max(0) as usize
    } else {
        (idx as usize).min(len)
    }
}

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

        let len = s.len();

        // Get optional start parameter (default: 0)
        let start = if args.len() > 2 {
            let start_val = args[2].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for start".to_owned()),
                )
            })? as i64;
            normalize_index(start_val, len)
        } else {
            0
        };

        // Get optional end parameter (default: string length)
        let end = if args.len() > 3 {
            let end_val = args[3].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for end".to_owned()),
                )
            })? as i64;
            normalize_index(end_val, len)
        } else {
            len
        };

        // Search within the slice [start, end)
        if start >= end || start >= len {
            return Ok(Rc::new(Variable::Null));
        }

        let slice = &s[start..end.min(len)];
        match slice.find(search) {
            Some(idx) => Ok(Rc::new(Variable::Number(serde_json::Number::from(
                (start + idx) as i64,
            )))),
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// last_index_of(string, search, start?, end?) -> number | null (JEP-014)
// =============================================================================

define_function!(
    LastIndexOfFn,
    vec![ArgumentType::String, ArgumentType::String],
    Some(ArgumentType::Number)
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

        let len = s.len();

        // Get optional start parameter (default: 0)
        let start = if args.len() > 2 {
            let start_val = args[2].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for start".to_owned()),
                )
            })? as i64;
            normalize_index(start_val, len)
        } else {
            0
        };

        // Get optional end parameter (default: string length)
        let end = if args.len() > 3 {
            let end_val = args[3].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for end".to_owned()),
                )
            })? as i64;
            normalize_index(end_val, len)
        } else {
            len
        };

        // Search within the slice [start, end)
        if start >= end || start >= len {
            return Ok(Rc::new(Variable::Null));
        }

        let slice = &s[start..end.min(len)];
        match slice.rfind(search) {
            Some(idx) => Ok(Rc::new(Variable::Number(serde_json::Number::from(
                (start + idx) as i64,
            )))),
            None => Ok(Rc::new(Variable::Null)),
        }
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
// format(template, args) -> string
// Supports:
//   - Positional with array: format('Hello {0}', ['World'])
//   - Named with object: format('Hello {name}', {name: 'World'})
//   - Variadic: format('Hello {0}', 'World')
// =============================================================================

define_function!(
    FormatFn,
    vec![ArgumentType::String],
    Some(ArgumentType::Any)
);

/// Convert a Variable to its string representation for formatting
fn var_to_format_string(v: &Variable) -> String {
    match v {
        Variable::String(s) => s.clone(),
        Variable::Number(n) => n.to_string(),
        Variable::Bool(b) => b.to_string(),
        Variable::Null => "null".to_string(),
        _ => serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
    }
}

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

        // Check if second arg is an array or object for unified formatting
        if args.len() == 2 {
            if let Some(arr) = args[1].as_array() {
                // Array-based positional: format('Hello {0}', ['World'])
                for (i, item) in arr.iter().enumerate() {
                    let placeholder = format!("{{{}}}", i);
                    let value = var_to_format_string(item);
                    result = result.replace(&placeholder, &value);
                }
                return Ok(Rc::new(Variable::String(result)));
            } else if let Some(obj) = args[1].as_object() {
                // Object-based named: format('Hello {name}', {name: 'World'})
                for (key, val) in obj.iter() {
                    let placeholder = format!("{{{}}}", key);
                    let value = var_to_format_string(val);
                    result = result.replace(&placeholder, &value);
                }
                return Ok(Rc::new(Variable::String(result)));
            }
        }

        // Fallback: variadic arguments format('Hello {0}', 'World')
        for (i, arg) in args.iter().skip(1).enumerate() {
            let placeholder = format!("{{{}}}", i);
            let value = var_to_format_string(arg);
            result = result.replace(&placeholder, &value);
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// sprintf(format_string, ...args) -> string
// Printf-style formatting with %s, %d, %f, etc.
// =============================================================================

define_function!(
    SprintfFn,
    vec![ArgumentType::String],
    Some(ArgumentType::Any)
);

impl Function for SprintfFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let format_str = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected format string".to_owned()),
            )
        })?;

        // Get arguments - either from array or variadic
        let format_args: Vec<&Variable> = if args.len() == 2 {
            if let Some(arr) = args[1].as_array() {
                arr.iter().map(|v| v.as_ref()).collect()
            } else {
                args.iter().skip(1).map(|v| v.as_ref()).collect()
            }
        } else {
            args.iter().skip(1).map(|v| v.as_ref()).collect()
        };

        let mut result = String::new();
        let mut arg_index = 0;
        let mut chars = format_str.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(&next) = chars.peek() {
                    if next == '%' {
                        // Escaped %
                        result.push('%');
                        chars.next();
                        continue;
                    }

                    // Parse format specifier
                    let mut width = String::new();
                    let mut precision = String::new();
                    let mut in_precision = false;

                    // Parse width and precision
                    while let Some(&ch) = chars.peek() {
                        if ch == '.' {
                            in_precision = true;
                            chars.next();
                        } else if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                            if in_precision {
                                precision.push(ch);
                            } else {
                                width.push(ch);
                            }
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    // Get the format type
                    if let Some(fmt_type) = chars.next() {
                        if arg_index < format_args.len() {
                            let arg = format_args[arg_index];
                            arg_index += 1;

                            let formatted = match fmt_type {
                                's' => var_to_format_string(arg),
                                'd' | 'i' => {
                                    if let Some(n) = arg.as_number() {
                                        format!("{}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'f' => {
                                    if let Some(n) = arg.as_number() {
                                        let prec: usize = precision.parse().unwrap_or(6);
                                        format!("{:.prec$}", n, prec = prec)
                                    } else {
                                        "0.0".to_string()
                                    }
                                }
                                'e' => {
                                    if let Some(n) = arg.as_number() {
                                        let prec: usize = precision.parse().unwrap_or(6);
                                        format!("{:.prec$e}", n, prec = prec)
                                    } else {
                                        "0e0".to_string()
                                    }
                                }
                                'x' => {
                                    if let Some(n) = arg.as_number() {
                                        format!("{:x}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'X' => {
                                    if let Some(n) = arg.as_number() {
                                        format!("{:X}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'o' => {
                                    if let Some(n) = arg.as_number() {
                                        format!("{:o}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'b' => {
                                    if let Some(n) = arg.as_number() {
                                        format!("{:b}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'c' => {
                                    if let Some(n) = arg.as_number() {
                                        char::from_u32(n as u32)
                                            .map(|c| c.to_string())
                                            .unwrap_or_default()
                                    } else if let Some(s) = arg.as_string() {
                                        s.chars().next().map(|c| c.to_string()).unwrap_or_default()
                                    } else {
                                        String::new()
                                    }
                                }
                                _ => {
                                    // Unknown format, just output as-is
                                    format!("%{}{}", width, fmt_type)
                                }
                            };

                            // Apply width if specified
                            if !width.is_empty() {
                                let w: i32 = width.parse().unwrap_or(0);
                                if w < 0 {
                                    // Left-align
                                    result.push_str(&format!(
                                        "{:<width$}",
                                        formatted,
                                        width = w.unsigned_abs() as usize
                                    ));
                                } else {
                                    // Right-align
                                    result.push_str(&format!(
                                        "{:>width$}",
                                        formatted,
                                        width = w as usize
                                    ));
                                }
                            } else {
                                result.push_str(&formatted);
                            }
                        } else {
                            // Not enough arguments, output placeholder
                            result.push('%');
                            result.push_str(&width);
                            if !precision.is_empty() {
                                result.push('.');
                                result.push_str(&precision);
                            }
                            result.push(fmt_type);
                        }
                    }
                } else {
                    // % at end of string
                    result.push('%');
                }
            } else {
                result.push(c);
            }
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// ltrimstr(string, prefix) -> string
// =============================================================================

define_function!(
    LtrimstrFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for LtrimstrFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let prefix = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected prefix string".to_owned()),
            )
        })?;

        let result = s.strip_prefix(prefix).unwrap_or(s).to_string();
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// rtrimstr(string, suffix) -> string
// =============================================================================

define_function!(
    RtrimstrFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for RtrimstrFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let suffix = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected suffix string".to_owned()),
            )
        })?;

        let result = s.strip_suffix(suffix).unwrap_or(s).to_string();
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// indices(string, search) -> array of indices
// =============================================================================

define_function!(
    IndicesFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for IndicesFn {
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

        // Find all indices (including overlapping matches)
        let mut indices: Vec<Rcvar> = Vec::new();
        if !search.is_empty() {
            let mut start = 0;
            while let Some(pos) = s[start..].find(search) {
                let actual_pos = start + pos;
                indices.push(Rc::new(Variable::Number(serde_json::Number::from(
                    actual_pos as i64,
                ))));
                start = actual_pos + 1; // Move by 1 to find overlapping matches
            }
        }

        Ok(Rc::new(Variable::Array(indices)))
    }
}

// =============================================================================
// inside(search, string) -> boolean
// =============================================================================

define_function!(
    InsideFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for InsideFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let search = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected search string".to_owned()),
            )
        })?;

        let s = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Bool(s.contains(search))))
    }
}

// =============================================================================
// humanize(string) -> string
// Converts a camelCase, snake_case, or kebab-case string to a human-readable form
// =============================================================================

define_function!(HumanizeFn, vec![ArgumentType::String], None);

impl Function for HumanizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Split on underscores, hyphens, and camelCase boundaries
        let mut result = String::new();
        let mut prev_was_lower = false;
        let mut word_start = true;

        for c in s.chars() {
            if c == '_' || c == '-' {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                word_start = true;
                prev_was_lower = false;
            } else if c.is_uppercase() && prev_was_lower {
                // camelCase boundary
                result.push(' ');
                if word_start {
                    result.push(c); // Keep first letter of sentence uppercase
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                word_start = false;
                prev_was_lower = false;
            } else {
                if word_start && result.is_empty() {
                    // First character of the string - capitalize it
                    result.push(c.to_uppercase().next().unwrap_or(c));
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                prev_was_lower = c.is_lowercase();
                word_start = false;
            }
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// deburr(string) -> string
// Removes diacritical marks (accents) from characters
// =============================================================================

define_function!(DeburrrFn, vec![ArgumentType::String], None);

impl Function for DeburrrFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Remove diacritical marks by mapping accented characters to ASCII
        let result: String = s
            .chars()
            .map(|c| match c {
                'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => 'A',
                'Æ' => 'A', // Could be "AE" but keeping single char
                'Ç' => 'C',
                'È' | 'É' | 'Ê' | 'Ë' => 'E',
                'Ì' | 'Í' | 'Î' | 'Ï' => 'I',
                'Ð' => 'D',
                'Ñ' => 'N',
                'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => 'O',
                'Ù' | 'Ú' | 'Û' | 'Ü' => 'U',
                'Ý' => 'Y',
                'Þ' => 'T', // Thorn
                'ß' => 's', // German sharp s
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
                'æ' => 'a', // Could be "ae" but keeping single char
                'ç' => 'c',
                'è' | 'é' | 'ê' | 'ë' => 'e',
                'ì' | 'í' | 'î' | 'ï' => 'i',
                'ð' => 'd',
                'ñ' => 'n',
                'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
                'ù' | 'ú' | 'û' | 'ü' => 'u',
                'ý' | 'ÿ' => 'y',
                'þ' => 't', // Thorn
                _ => c,
            })
            .collect();

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// words(string) -> array
// Splits string into an array of words
// =============================================================================

define_function!(WordsFn, vec![ArgumentType::String], None);

impl Function for WordsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Split on word boundaries: spaces, underscores, hyphens, and camelCase
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut prev_was_lower = false;

        for c in s.chars() {
            if c.is_whitespace() || c == '_' || c == '-' {
                if !current_word.is_empty() {
                    words.push(Rc::new(Variable::String(current_word.clone())) as Rcvar);
                    current_word.clear();
                }
                prev_was_lower = false;
            } else if c.is_uppercase() && prev_was_lower {
                // camelCase boundary
                if !current_word.is_empty() {
                    words.push(Rc::new(Variable::String(current_word.clone())) as Rcvar);
                    current_word.clear();
                }
                current_word.push(c);
                prev_was_lower = false;
            } else {
                current_word.push(c);
                prev_was_lower = c.is_lowercase();
            }
        }

        if !current_word.is_empty() {
            words.push(Rc::new(Variable::String(current_word)) as Rcvar);
        }

        Ok(Rc::new(Variable::Array(words)))
    }
}

// =============================================================================
// escape(string) -> string
// Escapes HTML entities: &, <, >, ", '
// =============================================================================

define_function!(EscapeFn, vec![ArgumentType::String], None);

impl Function for EscapeFn {
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
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;");

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// unescape(string) -> string
// Unescapes HTML entities: &amp;, &lt;, &gt;, &quot;, &#39;
// =============================================================================

define_function!(UnescapeFn, vec![ArgumentType::String], None);

impl Function for UnescapeFn {
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
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// escape_regex(string) -> string
// Escapes special regex characters
// =============================================================================

define_function!(EscapeRegexFn, vec![ArgumentType::String], None);

impl Function for EscapeRegexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Escape regex special characters: \ ^ $ . | ? * + ( ) [ ] { }
        let mut result = String::with_capacity(s.len() * 2);
        for c in s.chars() {
            match c {
                '\\' | '^' | '$' | '.' | '|' | '?' | '*' | '+' | '(' | ')' | '[' | ']' | '{'
                | '}' => {
                    result.push('\\');
                    result.push(c);
                }
                _ => result.push(c),
            }
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// start_case(string) -> string
// Converts string to Start Case (capitalize first letter of each word)
// =============================================================================

define_function!(StartCaseFn, vec![ArgumentType::String], None);

impl Function for StartCaseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Split on word boundaries and capitalize each word
        let mut result = String::new();
        let mut prev_was_lower = false;
        let mut word_start = true;

        for c in s.chars() {
            if c.is_whitespace() || c == '_' || c == '-' {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                word_start = true;
                prev_was_lower = false;
            } else if c.is_uppercase() && prev_was_lower {
                // camelCase boundary - start new word
                result.push(' ');
                result.push(c); // Keep uppercase
                word_start = false;
                prev_was_lower = false;
            } else {
                if word_start {
                    result.push(c.to_uppercase().next().unwrap_or(c));
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                prev_was_lower = c.is_lowercase();
                word_start = false;
            }
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// mask(string, visible?, char?) -> string
// Mask a string, optionally keeping the last N characters visible
// =============================================================================

define_function!(MaskFn, vec![ArgumentType::String], Some(ArgumentType::Any));

impl Function for MaskFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Number of characters to keep visible at the end (default: 0)
        let visible = if args.len() > 1 && !args[1].is_null() {
            args[1].as_number().unwrap_or(0.0) as usize
        } else {
            0
        };

        // Mask character (default: '*')
        let mask_char = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_string()
                .and_then(|s| s.chars().next())
                .unwrap_or('*')
        } else {
            '*'
        };

        let char_count = s.chars().count();

        if visible >= char_count {
            // If visible >= length, return original string
            return Ok(Rc::new(Variable::String(s.to_string())));
        }

        let mask_count = char_count - visible;
        let masked: String = std::iter::repeat_n(mask_char, mask_count)
            .chain(s.chars().skip(mask_count))
            .collect();

        Ok(Rc::new(Variable::String(masked)))
    }
}

// =============================================================================
// redact(string, pattern, replacement?) -> string
// Replace all matches of a regex pattern with a replacement string
// =============================================================================

#[cfg(feature = "regex")]
define_function!(
    RedactFn,
    vec![ArgumentType::String, ArgumentType::String],
    Some(ArgumentType::Any)
);

#[cfg(feature = "regex")]
impl Function for RedactFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let pattern = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string pattern".to_owned()),
            )
        })?;

        // Replacement string (default: "[REDACTED]")
        let replacement = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "[REDACTED]".to_string())
        } else {
            "[REDACTED]".to_string()
        };

        let re = regex::Regex::new(pattern).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse(format!("Invalid regex pattern: {}", e)),
            )
        })?;

        let result = re.replace_all(s, replacement.as_str());
        Ok(Rc::new(Variable::String(result.into_owned())))
    }
}

// =============================================================================
// normalize_whitespace(string) -> string
// Collapse multiple whitespace characters into a single space
// =============================================================================

define_function!(NormalizeWhitespaceFn, vec![ArgumentType::String], None);

impl Function for NormalizeWhitespaceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Split on whitespace and rejoin with single spaces
        let result: String = s.split_whitespace().collect::<Vec<_>>().join(" ");

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// is_blank(string) -> boolean
// Check if string is empty or contains only whitespace
// =============================================================================

define_function!(IsBlankFn, vec![ArgumentType::String], None);

impl Function for IsBlankFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let is_blank = s.trim().is_empty();
        Ok(Rc::new(Variable::Bool(is_blank)))
    }
}

// =============================================================================
// abbreviate(string, max_length, suffix?) -> string
// Truncate string to max length with ellipsis suffix
// =============================================================================

define_function!(
    AbbreviateFn,
    vec![ArgumentType::String, ArgumentType::Number],
    Some(ArgumentType::Any)
);

impl Function for AbbreviateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let max_length = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for max_length".to_owned()),
            )
        })? as usize;

        // Suffix (default: "...")
        let suffix = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "...".to_string())
        } else {
            "...".to_string()
        };

        let char_count = s.chars().count();
        let suffix_len = suffix.chars().count();

        if char_count <= max_length {
            return Ok(Rc::new(Variable::String(s.to_string())));
        }

        if max_length <= suffix_len {
            // If max_length is too small for suffix, just truncate
            let result: String = s.chars().take(max_length).collect();
            return Ok(Rc::new(Variable::String(result)));
        }

        let truncate_at = max_length - suffix_len;
        let mut result: String = s.chars().take(truncate_at).collect();
        result.push_str(&suffix);

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// center(string, width, char?) -> string
// Center-pad a string to the given width
// =============================================================================

define_function!(
    CenterFn,
    vec![ArgumentType::String, ArgumentType::Number],
    Some(ArgumentType::Any)
);

impl Function for CenterFn {
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

        // Padding character (default: ' ')
        let pad_char = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_string()
                .and_then(|s| s.chars().next())
                .unwrap_or(' ')
        } else {
            ' '
        };

        let char_count = s.chars().count();

        if char_count >= width {
            return Ok(Rc::new(Variable::String(s.to_string())));
        }

        let total_padding = width - char_count;
        let left_padding = total_padding / 2;
        let right_padding = total_padding - left_padding;

        let mut result = String::with_capacity(width);
        for _ in 0..left_padding {
            result.push(pad_char);
        }
        result.push_str(s);
        for _ in 0..right_padding {
            result.push(pad_char);
        }

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// reverse_string(string) -> string
// Reverse a string (Unicode-aware, reverses grapheme clusters ideally, chars for simplicity)
// =============================================================================

define_function!(ReverseStringFn, vec![ArgumentType::String], None);

impl Function for ReverseStringFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let result: String = s.chars().rev().collect();
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

    #[test]
    fn test_ltrimstr() {
        let runtime = setup_runtime();
        let expr = runtime.compile("ltrimstr(@, 'hello ')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "world");
    }

    #[test]
    fn test_ltrimstr_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("ltrimstr(@, 'foo')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_rtrimstr() {
        let runtime = setup_runtime();
        let expr = runtime.compile("rtrimstr(@, ' world')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_rtrimstr_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("rtrimstr(@, 'foo')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_indices() {
        let runtime = setup_runtime();
        let expr = runtime.compile("indices(@, 'l')").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number().unwrap() as i64, 2);
        assert_eq!(arr[1].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_indices_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("indices(@, 'x')").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_indices_overlapping() {
        let runtime = setup_runtime();
        let expr = runtime.compile("indices(@, 'aa')").unwrap();
        let data = Variable::String("aaa".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number().unwrap() as i64, 0);
        assert_eq!(arr[1].as_number().unwrap() as i64, 1);
    }

    #[test]
    fn test_inside() {
        let runtime = setup_runtime();
        let expr = runtime.compile("inside('world', @)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_inside_not_found() {
        let runtime = setup_runtime();
        let expr = runtime.compile("inside('foo', @)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_format_with_array() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("format('Hello {0}, you have {1} messages', @)")
            .unwrap();
        let data: Variable = serde_json::from_str(r#"["Alice", 5]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "Hello Alice, you have 5 messages"
        );
    }

    #[test]
    fn test_format_with_object() {
        let runtime = setup_runtime();
        let expr = runtime.compile("format('Hello {name}!', @)").unwrap();
        let data: Variable = serde_json::from_str(r#"{"name": "World"}"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello World!");
    }

    #[test]
    fn test_sprintf_string() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('Hello, %s!', @)").unwrap();
        let data = Variable::String("World".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_sprintf_integer() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('%d + %d = %d', @)").unwrap();
        let data: Variable = serde_json::from_str("[1, 2, 3]").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "1 + 2 = 3");
    }

    #[test]
    fn test_sprintf_float_precision() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('Pi is %.2f', @)").unwrap();
        let data: Variable = serde_json::from_str("3.14159").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Pi is 3.14");
    }

    #[test]
    fn test_sprintf_hex() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('Hex: %x', @)").unwrap();
        let data: Variable = serde_json::from_str("255").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hex: ff");
    }

    #[test]
    fn test_sprintf_width() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('%10s', @)").unwrap();
        let data = Variable::String("hi".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "        hi");
    }

    #[test]
    fn test_sprintf_escaped_percent() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('100%% done', @)").unwrap();
        let data = Variable::Null;
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "100% done");
    }

    #[test]
    fn test_humanize_snake_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("humanize(@)").unwrap();
        let data = Variable::String("hello_world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello world");
    }

    #[test]
    fn test_humanize_camel_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("humanize(@)").unwrap();
        let data = Variable::String("helloWorld".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello world");
    }

    #[test]
    fn test_humanize_kebab_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("humanize(@)").unwrap();
        let data = Variable::String("hello-world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello world");
    }

    #[test]
    fn test_deburr() {
        let runtime = setup_runtime();
        let expr = runtime.compile("deburr(@)").unwrap();
        let data = Variable::String("déjà vu".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "deja vu");
    }

    #[test]
    fn test_deburr_accents() {
        let runtime = setup_runtime();
        let expr = runtime.compile("deburr(@)").unwrap();
        let data = Variable::String("àéîõü".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "aeiou");
    }

    #[test]
    fn test_words() {
        let runtime = setup_runtime();
        let expr = runtime.compile("words(@)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "hello");
        assert_eq!(arr[1].as_string().unwrap(), "world");
    }

    #[test]
    fn test_words_camel_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("words(@)").unwrap();
        let data = Variable::String("helloWorld".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "hello");
        assert_eq!(arr[1].as_string().unwrap(), "World");
    }

    #[test]
    fn test_escape() {
        let runtime = setup_runtime();
        let expr = runtime.compile("escape(@)").unwrap();
        let data = Variable::String("<div class=\"test\">Hello & World</div>".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "&lt;div class=&quot;test&quot;&gt;Hello &amp; World&lt;/div&gt;"
        );
    }

    #[test]
    fn test_unescape() {
        let runtime = setup_runtime();
        let expr = runtime.compile("unescape(@)").unwrap();
        let data = Variable::String("&lt;div&gt;Hello &amp; World&lt;/div&gt;".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "<div>Hello & World</div>");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let runtime = setup_runtime();
        let expr = runtime.compile("unescape(escape(@))").unwrap();
        let data = Variable::String("<div>Hello & World</div>".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "<div>Hello & World</div>");
    }

    #[test]
    fn test_escape_regex() {
        let runtime = setup_runtime();
        let expr = runtime.compile("escape_regex(@)").unwrap();
        let data = Variable::String("hello.world?".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello\\.world\\?");
    }

    #[test]
    fn test_escape_regex_special_chars() {
        let runtime = setup_runtime();
        let expr = runtime.compile("escape_regex(@)").unwrap();
        let data = Variable::String("[a-z]+(foo|bar)*".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "\\[a-z\\]\\+\\(foo\\|bar\\)\\*"
        );
    }

    #[test]
    fn test_start_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("start_case(@)").unwrap();
        let data = Variable::String("hello_world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello World");
    }

    #[test]
    fn test_start_case_camel() {
        let runtime = setup_runtime();
        let expr = runtime.compile("start_case(@)").unwrap();
        let data = Variable::String("helloWorld".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello World");
    }

    // JEP-014 find_first tests
    #[test]
    fn test_find_first_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'world')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 6);
    }

    #[test]
    fn test_find_first_not_found() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'xyz')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_find_first_with_start() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'o', `5`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 7);
    }

    #[test]
    fn test_find_first_with_start_and_end() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'o', `0`, `5`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 4);
    }

    #[test]
    fn test_find_first_with_negative_start() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'o', `-5`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 7);
    }

    // JEP-014 find_last tests
    #[test]
    fn test_find_last_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'o')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 7);
    }

    #[test]
    fn test_find_last_not_found() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'xyz')").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_find_last_with_start_and_end() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'o', `0`, `6`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 4);
    }

    #[test]
    fn test_find_last_with_negative_end() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'l', `0`, `-1`)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 9);
    }

    // =========================================================================
    // mask tests
    // =========================================================================

    #[test]
    fn test_mask_all() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@)").unwrap();
        let data = Variable::String("secret".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "******");
    }

    #[test]
    fn test_mask_keep_last_4() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@, `4`)").unwrap();
        let data = Variable::String("4111111111111111".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "************1111");
    }

    #[test]
    fn test_mask_custom_char() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@, `0`, `\"X\"`)").unwrap();
        let data = Variable::String("secret".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "XXXXXX");
    }

    #[test]
    fn test_mask_visible_exceeds_length() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@, `10`)").unwrap();
        let data = Variable::String("short".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "short");
    }

    // =========================================================================
    // redact tests (requires regex feature)
    // =========================================================================

    #[test]
    #[cfg(feature = "regex")]
    fn test_redact_email() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile(
                r#"redact(@, `"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"`, `"[EMAIL]"`)"#,
            )
            .unwrap();
        let data = Variable::String("Contact us at test@example.com for help".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "Contact us at [EMAIL] for help"
        );
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_redact_default_replacement() {
        let runtime = setup_runtime();
        let expr = runtime.compile(r#"redact(@, `"secret"` )"#).unwrap();
        let data = Variable::String("The secret password is secret".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "The [REDACTED] password is [REDACTED]"
        );
    }

    // =========================================================================
    // normalize_whitespace tests
    // =========================================================================

    #[test]
    fn test_normalize_whitespace_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize_whitespace(@)").unwrap();
        let data = Variable::String("hello    world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_normalize_whitespace_mixed() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize_whitespace(@)").unwrap();
        let data = Variable::String("hello\t\n  world\n\nfoo".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world foo");
    }

    #[test]
    fn test_normalize_whitespace_leading_trailing() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize_whitespace(@)").unwrap();
        let data = Variable::String("  hello world  ".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    // =========================================================================
    // is_blank tests
    // =========================================================================

    #[test]
    fn test_is_blank_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_blank(@)").unwrap();
        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_blank_whitespace() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_blank(@)").unwrap();
        let data = Variable::String("   \t\n  ".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_blank_not_blank() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_blank(@)").unwrap();
        let data = Variable::String("  a  ".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    // =========================================================================
    // abbreviate tests
    // =========================================================================

    #[test]
    fn test_abbreviate_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("abbreviate(@, `10`)").unwrap();
        let data = Variable::String("This is a very long string".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "This is...");
    }

    #[test]
    fn test_abbreviate_no_truncation() {
        let runtime = setup_runtime();
        let expr = runtime.compile("abbreviate(@, `20`)").unwrap();
        let data = Variable::String("short".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "short");
    }

    #[test]
    fn test_abbreviate_custom_suffix() {
        let runtime = setup_runtime();
        let expr = runtime.compile("abbreviate(@, `8`, `\"~\"`)").unwrap();
        let data = Variable::String("Hello World".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello W~");
    }

    // =========================================================================
    // center tests
    // =========================================================================

    #[test]
    fn test_center_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `10`)").unwrap();
        let data = Variable::String("hi".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "    hi    ");
    }

    #[test]
    fn test_center_custom_char() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `10`, `\"-\"`)").unwrap();
        let data = Variable::String("hi".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "----hi----");
    }

    #[test]
    fn test_center_already_wide() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `3`)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_center_odd_padding() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `7`)").unwrap();
        let data = Variable::String("hi".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "  hi   ");
    }

    // =========================================================================
    // reverse_string tests
    // =========================================================================

    #[test]
    fn test_reverse_string_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("reverse_string(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "olleh");
    }

    #[test]
    fn test_reverse_string_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("reverse_string(@)").unwrap();
        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "");
    }

    #[test]
    fn test_reverse_string_palindrome() {
        let runtime = setup_runtime();
        let expr = runtime.compile("reverse_string(@)").unwrap();
        let data = Variable::String("racecar".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "racecar");
    }
}
