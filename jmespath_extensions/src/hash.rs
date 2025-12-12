//! Hash and checksum functions.
//!
//! This module provides cryptographic hash, HMAC, and checksum functions for JMESPath expressions.
//! It includes support for MD5, SHA-1, SHA-256, SHA-512 cryptographic hashes, HMAC variants,
//! and CRC32 checksums. All hash functions return hexadecimal string representations except
//! CRC32 which returns a numeric checksum.
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
//! | `sha512` | `(text: string)` | `string` | Compute SHA-512 hash (hex-encoded) |
//! | `hmac_md5` | `(text: string, key: string)` | `string` | Compute HMAC-MD5 (hex-encoded) |
//! | `hmac_sha1` | `(text: string, key: string)` | `string` | Compute HMAC-SHA1 (hex-encoded) |
//! | `hmac_sha256` | `(text: string, key: string)` | `string` | Compute HMAC-SHA256 (hex-encoded) |
//! | `hmac_sha512` | `(text: string, key: string)` | `string` | Compute HMAC-SHA512 (hex-encoded) |
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
//! ### `sha512(text: string) -> string`
//!
//! Computes the SHA-512 hash of the input string and returns it as a lowercase hexadecimal string.
//! SHA-512 is part of the SHA-2 family and provides stronger security than SHA-256.
//!
//! ```text
//! sha512('hello')                          // "9b71d224bd62f3785d96d46ad3ea3d73..."
//! sha512('')                               // "cf83e1357eefb8bdf1542850d66d8007..."
//! ```
//!
//! ## HMAC Functions
//!
//! HMAC (Hash-based Message Authentication Code) functions compute a keyed hash that can be
//! used to verify both data integrity and authenticity. Common use cases include:
//! - Webhook signature verification (GitHub, Stripe, etc.)
//! - API request signing
//! - Token generation
//!
//! ### `hmac_md5(text: string, key: string) -> string`
//!
//! Computes HMAC-MD5 of the input using the provided key.
//!
//! **Warning:** HMAC-MD5 is considered weak. Use HMAC-SHA256 or HMAC-SHA512 for new applications.
//!
//! ```text
//! hmac_md5('hello', 'secret')              // "e17e4e4a2ef59e02498b1f1e4c1b7272"
//! ```
//!
//! ### `hmac_sha1(text: string, key: string) -> string`
//!
//! Computes HMAC-SHA1 of the input using the provided key.
//! Still used by some APIs (e.g., OAuth 1.0, older webhook implementations).
//!
//! ```text
//! hmac_sha1('hello', 'secret')             // "5112055c05f944f85755efc5cd8970e194e9f45b"
//! ```
//!
//! ### `hmac_sha256(text: string, key: string) -> string`
//!
//! Computes HMAC-SHA256 of the input using the provided key.
//! This is the recommended HMAC algorithm for most applications.
//!
//! ```text
//! hmac_sha256('hello', 'secret')           // "88aab3ede8d3adf94d26ab90d3bafd4a..."
//! // Verify a GitHub webhook signature:
//! hmac_sha256(payload, webhook_secret) == headers.`"X-Hub-Signature-256"`
//! ```
//!
//! ### `hmac_sha512(text: string, key: string) -> string`
//!
//! Computes HMAC-SHA512 of the input using the provided key.
//! Provides maximum security when needed.
//!
//! ```text
//! hmac_sha512('hello', 'secret')           // "d05888a201606a6979..."
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
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

// Type aliases for HMAC variants
type HmacMd5 = Hmac<Md5>;
type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

/// Register all hash functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    // Hash functions
    runtime.register_function("md5", Box::new(Md5Fn::new()));
    runtime.register_function("sha1", Box::new(Sha1Fn::new()));
    runtime.register_function("sha256", Box::new(Sha256Fn::new()));
    runtime.register_function("sha512", Box::new(Sha512Fn::new()));

    // HMAC functions
    runtime.register_function("hmac_md5", Box::new(HmacMd5Fn::new()));
    runtime.register_function("hmac_sha1", Box::new(HmacSha1Fn::new()));
    runtime.register_function("hmac_sha256", Box::new(HmacSha256Fn::new()));
    runtime.register_function("hmac_sha512", Box::new(HmacSha512Fn::new()));

    // Checksum functions
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
// sha512(string) -> string (hex-encoded SHA-512 hash)
// =============================================================================

define_function!(Sha512Fn, vec![ArgumentType::String], None);

impl Function for Sha512Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Sha512::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// hmac_md5(text, key) -> string (hex-encoded HMAC-MD5)
// =============================================================================

define_function!(
    HmacMd5Fn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for HmacMd5Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacMd5::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// hmac_sha1(text, key) -> string (hex-encoded HMAC-SHA1)
// =============================================================================

define_function!(
    HmacSha1Fn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for HmacSha1Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// hmac_sha256(text, key) -> string (hex-encoded HMAC-SHA256)
// =============================================================================

define_function!(
    HmacSha256Fn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for HmacSha256Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Rc::new(Variable::String(hex_string)))
    }
}

// =============================================================================
// hmac_sha512(text, key) -> string (hex-encoded HMAC-SHA512)
// =============================================================================

define_function!(
    HmacSha512Fn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for HmacSha512Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacSha512::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

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

    // =========================================================================
    // Hash function tests
    // =========================================================================

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
    fn test_md5_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("md5(@)").unwrap();
        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
    }

    #[test]
    fn test_sha1() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha1(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
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

    #[test]
    fn test_sha512() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha512(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
        );
    }

    #[test]
    fn test_sha512_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha512(@)").unwrap();
        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
        );
    }

    // =========================================================================
    // HMAC function tests
    // =========================================================================

    #[test]
    fn test_hmac_md5() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_md5(@, `\"secret\"`)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        // Verified against Python hmac module
        assert_eq!(
            result.as_string().unwrap(),
            "bade63863c61ed0b3165806ecd6acefc"
        );
    }

    #[test]
    fn test_hmac_sha1() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha1(@, `\"secret\"`)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        // Verified against external HMAC-SHA1 calculator
        assert_eq!(
            result.as_string().unwrap(),
            "5112055c05f944f85755efc5cd8970e194e9f45b"
        );
    }

    #[test]
    fn test_hmac_sha256() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha256(@, `\"secret\"`)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        // Verified against Python hmac module
        assert_eq!(
            result.as_string().unwrap(),
            "88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b"
        );
    }

    #[test]
    fn test_hmac_sha512() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha512(@, `\"secret\"`)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        // Verified against Python hmac module
        assert_eq!(
            result.as_string().unwrap(),
            "db1595ae88a62fd151ec1cba81b98c39df82daae7b4cb9820f446d5bf02f1dcfca6683d88cab3e273f5963ab8ec469a746b5b19086371239f67d1e5f99a79440"
        );
    }

    #[test]
    fn test_hmac_sha256_empty_message() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha256(@, `\"key\"`)").unwrap();
        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        // HMAC of empty string with key "key"
        assert_eq!(
            result.as_string().unwrap(),
            "5d5d139563c95b5967b9bd9a8c9b233a9dedb45072794cd232dc1b74832607d0"
        );
    }

    #[test]
    fn test_hmac_sha256_empty_key() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha256(@, `\"\"`)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        // HMAC with empty key is valid
        assert!(!result.as_string().unwrap().is_empty());
    }

    // =========================================================================
    // CRC32 tests
    // =========================================================================

    #[test]
    fn test_crc32() {
        let runtime = setup_runtime();
        let expr = runtime.compile("crc32(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as u64, 907060870);
    }

    #[test]
    fn test_crc32_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("crc32(@)").unwrap();
        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as u64, 0);
    }
}
