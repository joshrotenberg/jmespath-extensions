//! Common types and utilities for JMESPath extension functions.

use std::rc::Rc;

pub use jmespath::functions::{ArgumentType, Function, Signature};
pub use jmespath::{Context, ErrorReason, JmespathError, Rcvar, Runtime, Variable};

/// Helper macro for defining JMESPath custom functions.
///
/// This macro creates a struct with a signature field and implements
/// the necessary boilerplate for creating new JMESPath functions.
#[macro_export]
macro_rules! define_function {
    ($name:ident, $args:expr, $variadic:expr) => {
        pub struct $name {
            signature: $crate::common::Signature,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    signature: $crate::common::Signature::new($args, $variadic),
                }
            }
        }
    };
}

/// Helper to create an Rcvar from a Variable
#[inline]
pub fn rcvar(v: Variable) -> Rcvar {
    Rc::new(v)
}
