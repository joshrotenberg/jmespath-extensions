//! Network/IP functions for JMESPath.
//!
//! This module provides functions for working with IP addresses and CIDR notation.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `ip_to_int(s)` | Convert IPv4 address to integer |
//! | `int_to_ip(n)` | Convert integer to IPv4 address |
//! | `cidr_contains(cidr, ip)` | Check if IP is within CIDR range |
//! | `cidr_network(cidr)` | Get network address of CIDR |
//! | `cidr_broadcast(cidr)` | Get broadcast address of CIDR |
//! | `cidr_prefix(cidr)` | Get prefix length of CIDR |
//! | `is_private_ip(ip)` | Check if IP is in private range |
//!
//! # Examples
//!
//! ```
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::network;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! network::register(&mut runtime);
//!
//! // Check if IP is in CIDR range
//! let data = Variable::from_json(r#"{"cidr": "192.168.1.0/24", "ip": "192.168.1.100"}"#).unwrap();
//! let expr = runtime.compile("cidr_contains(cidr, ip)").unwrap();
//! let result = expr.search(&data).unwrap();
//! // Result: true
//! ```

use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;

use ipnetwork::{IpNetwork, Ipv4Network};

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Signature, Variable};

/// Register all network functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("ip_to_int", Box::new(IpToIntFn::new()));
    runtime.register_function("int_to_ip", Box::new(IntToIpFn::new()));
    runtime.register_function("cidr_contains", Box::new(CidrContainsFn::new()));
    runtime.register_function("cidr_network", Box::new(CidrNetworkFn::new()));
    runtime.register_function("cidr_broadcast", Box::new(CidrBroadcastFn::new()));
    runtime.register_function("cidr_prefix", Box::new(CidrPrefixFn::new()));
    runtime.register_function("is_private_ip", Box::new(IsPrivateIpFn::new()));
}

// =============================================================================
// ip_to_int(s) -> number
// =============================================================================

pub struct IpToIntFn {
    signature: Signature,
}

impl Default for IpToIntFn {
    fn default() -> Self {
        Self::new()
    }
}

impl IpToIntFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for IpToIntFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        match Ipv4Addr::from_str(s) {
            Ok(ip) => {
                let int_val: u32 = ip.into();
                Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(int_val as f64).unwrap(),
                )))
            }
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// int_to_ip(n) -> string
// =============================================================================

pub struct IntToIpFn {
    signature: Signature,
}

impl Default for IntToIpFn {
    fn default() -> Self {
        Self::new()
    }
}

impl IntToIpFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::Number], None),
        }
    }
}

impl Function for IntToIpFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().unwrap();

        if n < 0.0 || n > u32::MAX as f64 {
            return Ok(Rc::new(Variable::Null));
        }

        let ip = Ipv4Addr::from(n as u32);
        Ok(Rc::new(Variable::String(ip.to_string())))
    }
}

// =============================================================================
// cidr_contains(cidr, ip) -> bool
// =============================================================================

pub struct CidrContainsFn {
    signature: Signature,
}

impl Default for CidrContainsFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CidrContainsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::String], None),
        }
    }
}

impl Function for CidrContainsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_string().unwrap();
        let ip_str = args[1].as_string().unwrap();

        let network = match IpNetwork::from_str(cidr_str) {
            Ok(n) => n,
            Err(_) => return Ok(Rc::new(Variable::Null)),
        };

        let ip: std::net::IpAddr = match ip_str.parse() {
            Ok(ip) => ip,
            Err(_) => return Ok(Rc::new(Variable::Null)),
        };

        Ok(Rc::new(Variable::Bool(network.contains(ip))))
    }
}

// =============================================================================
// cidr_network(cidr) -> string
// =============================================================================

pub struct CidrNetworkFn {
    signature: Signature,
}

impl Default for CidrNetworkFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CidrNetworkFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CidrNetworkFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_string().unwrap();

        match Ipv4Network::from_str(cidr_str) {
            Ok(network) => Ok(Rc::new(Variable::String(network.network().to_string()))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// cidr_broadcast(cidr) -> string
// =============================================================================

pub struct CidrBroadcastFn {
    signature: Signature,
}

impl Default for CidrBroadcastFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CidrBroadcastFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CidrBroadcastFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_string().unwrap();

        match Ipv4Network::from_str(cidr_str) {
            Ok(network) => Ok(Rc::new(Variable::String(network.broadcast().to_string()))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// cidr_prefix(cidr) -> number
// =============================================================================

pub struct CidrPrefixFn {
    signature: Signature,
}

impl Default for CidrPrefixFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CidrPrefixFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CidrPrefixFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_string().unwrap();

        match IpNetwork::from_str(cidr_str) {
            Ok(network) => Ok(Rc::new(Variable::Number(serde_json::Number::from(
                network.prefix(),
            )))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// is_private_ip(ip) -> bool
// =============================================================================

pub struct IsPrivateIpFn {
    signature: Signature,
}

impl Default for IsPrivateIpFn {
    fn default() -> Self {
        Self::new()
    }
}

impl IsPrivateIpFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for IsPrivateIpFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let ip_str = args[0].as_string().unwrap();

        match Ipv4Addr::from_str(ip_str) {
            Ok(ip) => Ok(Rc::new(Variable::Bool(ip.is_private()))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        runtime
    }

    #[test]
    fn test_ip_to_int() {
        let runtime = setup();
        let data = Variable::from_json(r#""192.168.1.1""#).unwrap();
        let expr = runtime.compile("ip_to_int(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // 192.168.1.1 = 192*256^3 + 168*256^2 + 1*256 + 1 = 3232235777
        assert_eq!(result.as_number().unwrap(), 3232235777.0);
    }

    #[test]
    fn test_int_to_ip() {
        let runtime = setup();
        let data = Variable::from_json(r#"3232235777"#).unwrap();
        let expr = runtime.compile("int_to_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "192.168.1.1");
    }

    #[test]
    fn test_ip_roundtrip() {
        let runtime = setup();
        let data = Variable::from_json(r#""10.0.0.1""#).unwrap();
        let expr = runtime.compile("int_to_ip(ip_to_int(@))").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "10.0.0.1");
    }

    #[test]
    fn test_cidr_contains_true() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"{"cidr": "192.168.1.0/24", "ip": "192.168.1.100"}"#).unwrap();
        let expr = runtime.compile("cidr_contains(cidr, ip)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_cidr_contains_false() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"{"cidr": "192.168.1.0/24", "ip": "192.168.2.1"}"#).unwrap();
        let expr = runtime.compile("cidr_contains(cidr, ip)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_cidr_network() {
        let runtime = setup();
        let data = Variable::from_json(r#""192.168.1.100/24""#).unwrap();
        let expr = runtime.compile("cidr_network(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "192.168.1.0");
    }

    #[test]
    fn test_cidr_broadcast() {
        let runtime = setup();
        let data = Variable::from_json(r#""192.168.1.0/24""#).unwrap();
        let expr = runtime.compile("cidr_broadcast(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "192.168.1.255");
    }

    #[test]
    fn test_cidr_prefix() {
        let runtime = setup();
        let data = Variable::from_json(r#""10.0.0.0/8""#).unwrap();
        let expr = runtime.compile("cidr_prefix(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_is_private_ip_true() {
        let runtime = setup();
        // 192.168.x.x is private
        let data = Variable::from_json(r#""192.168.1.1""#).unwrap();
        let expr = runtime.compile("is_private_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_private_ip_10() {
        let runtime = setup();
        // 10.x.x.x is private
        let data = Variable::from_json(r#""10.0.0.1""#).unwrap();
        let expr = runtime.compile("is_private_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_private_ip_false() {
        let runtime = setup();
        // 8.8.8.8 is public (Google DNS)
        let data = Variable::from_json(r#""8.8.8.8""#).unwrap();
        let expr = runtime.compile("is_private_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }
}
