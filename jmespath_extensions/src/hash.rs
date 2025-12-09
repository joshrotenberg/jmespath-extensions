//! Hash and checksum functions.
//!
//! This module provides cryptographic hash and checksum functions for JMESPath expressions.
//! It includes support for MD5, SHA-1, SHA-256 cryptographic hashes, and CRC32 checksums.
//! All hash functions return hexadecimal string representations except CRC32 which returns
//! a numeric checksum.
//!
//! **Note:** This module requires the `hash` feature to be enabled.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description |
//! |----------|-----------|---------|-------------|
//! | `md5` | `(text: string)` | `string` | Compute MD5 hash (hex-encoded) |
//! | `sha1` | `(text: string)` | `string` | Compute SHA-1 hash (hex-encoded) |
//! | `sha256` | `(text: string)` | `string` | Compute SHA-256 hash (hex-encoded) |
//! | `crc32` | `(text: string)` | `number` | Compute CRC32 checksum |
//!
//! # Examples
//!
//! ```rust
//! use jmespath_extensions::Runtime;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! jmespath_extensions::register_all(&mut runtime);
//!
//! let expr = runtime.compile("sha256(@)").unwrap();
//! let data = jmespath::Variable::String("hello".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(),
//!            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
//! ```
//!
//! # Function Details
//!
//! ## Cryptographic Hash Functions
//!
//! ### `md5(text: string) -> string`
//!
//! Computes the MD5 hash of the input string and returns it as a lowercase hexadecimal string.
//!
//! **Warning:** MD5 is cryptographically broken and should not be used for security purposes.
//! It's suitable for checksums and non-security applications.
//!
//! ```text
//! md5('hello')                             // "5d41402abc4b2a76b9719d911017c592"
//! md5('Hello World')                       // "b10a8db164e0754105b7a99be72e3fe5"
//! md5('')                                  // "d41d8cd98f00b204e9800998ecf8427e"
//! ```
//!
//! ### `sha1(text: string) -> string`
//!
//! Computes the SHA-1 hash of the input string and returns it as a lowercase hexadecimal string.
//!
//! **Warning:** SHA-1 is considered weak and should not be used for security-critical applications.
//!
//! ```text
//! sha1('hello')                            // "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
//! sha1('Hello World')                      // "0a4d55a8d778e5022fab701977c5d840bbc486d0"
//! sha1('')                                 // "da39a3ee5e6b4b0d3255bfef95601890afd80709"
//! ```
//!
//! ### `sha256(text: string) -> string`
//!
//! Computes the SHA-256 hash of the input string and returns it as a lowercase hexadecimal string.
//! SHA-256 is part of the SHA-2 family and is suitable for security applications.
//!
//! ```text
//! sha256('hello')                          // "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
//! sha256('Hello World')                    // "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
//! sha256('')                               // "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
//! ```
//!
//! ## Checksum Functions
//!
//! ### `crc32(text: string) -> number`
//!
//! Computes the CRC32 checksum of the input string and returns it as an unsigned 32-bit integer.
//! CRC32 is useful for error detection but not for cryptographic purposes.
//!
//! ```text
//! crc32('hello')                           // 907060870
//! crc32('Hello World')                     // 1243066710
//! crc32('')                                // 0
//! crc32('test')                            // 3632233996
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

use crc32fast::Hasher as Crc32Hasher;
use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::Sha256;

/// Register all hash functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("md5", Box::new(Md5Fn::new()));
    runtime.register_function("sha1", Box::new(Sha1Fn::new()));
    runtime.register_function("sha256", Box::new(Sha256Fn::new()));
    runtime.register_function("crc32", Box::new(Crc32Fn::new()));
}

// =============================================================================
// md5(string) -> string (hex-encoded MD5 hash)
// =============================================================================

define_function!(Md5Fn, vec![ArgumentType::String], None);

impl Function for Md5Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Md5::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// sha1(string) -> string (hex-encoded SHA-1 hash)
// =============================================================================

define_function!(Sha1Fn, vec![ArgumentType::String], None);

impl Function for Sha1Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Sha1::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// sha256(string) -> string (hex-encoded SHA-256 hash)
// =============================================================================

define_function!(Sha256Fn, vec![ArgumentType::String], None);

impl Function for Sha256Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// crc32(string) -> number (CRC32 checksum as integer)
// =============================================================================

define_function!(Crc32Fn, vec![ArgumentType::String], None);

impl Function for Crc32Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Crc32Hasher::new();
        hasher.update(input.as_bytes());
        let checksum = hasher.finalize();

        Ok(Rc::new(Variable::Number(serde_json::Number::from(
            checksum,
        ))))
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
    fn test_md5() {
        let runtime = setup_runtime();
        let expr = runtime.compile("md5(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "5d41402abc4b2a76b9719d911017c592"
        );
    }

    #[test]
    fn test_sha256() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha256(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
