//! Geospatial functions for JMESPath.
//!
//! This module provides functions for calculating distances and bearings
//! between geographic coordinates.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `geo_distance(lat1, lon1, lat2, lon2)` | Distance in meters using Haversine formula |
//! | `geo_distance_km(lat1, lon1, lat2, lon2)` | Distance in kilometers |
//! | `geo_distance_miles(lat1, lon1, lat2, lon2)` | Distance in miles |
//! | `geo_bearing(lat1, lon1, lat2, lon2)` | Initial bearing in degrees (0-360) |
//!
//! # Examples
//!
//! ```
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::geo;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! geo::register(&mut runtime);
//!
//! // Distance between New York and London
//! let data = Variable::from_json(r#"{"ny": [40.7128, -74.0060], "london": [51.5074, -0.1278]}"#).unwrap();
//! let expr = runtime.compile("geo_distance_km(ny[0], ny[1], london[0], london[1])").unwrap();
//! let result = expr.search(&data).unwrap();
//! // Result: ~5570 km
//! ```

use std::rc::Rc;

use geoutils::Location;

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Signature, Variable};

/// Register all geo functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("geo_distance", Box::new(GeoDistanceFn::new()));
    runtime.register_function("geo_distance_km", Box::new(GeoDistanceKmFn::new()));
    runtime.register_function("geo_distance_miles", Box::new(GeoDistanceMilesFn::new()));
    runtime.register_function("geo_bearing", Box::new(GeoBearingFn::new()));
}

// =============================================================================
// geo_distance(lat1, lon1, lat2, lon2) -> number (meters)
// =============================================================================

pub struct GeoDistanceFn {
    signature: Signature,
}

impl Default for GeoDistanceFn {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoDistanceFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                ],
                None,
            ),
        }
    }
}

impl Function for GeoDistanceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_number().unwrap();
        let lon1 = args[1].as_number().unwrap();
        let lat2 = args[2].as_number().unwrap();
        let lon2 = args[3].as_number().unwrap();

        let loc1 = Location::new(lat1, lon1);
        let loc2 = Location::new(lat2, lon2);

        let distance = loc1.haversine_distance_to(&loc2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(distance.meters()).unwrap(),
        )))
    }
}

// =============================================================================
// geo_distance_km(lat1, lon1, lat2, lon2) -> number (kilometers)
// =============================================================================

pub struct GeoDistanceKmFn {
    signature: Signature,
}

impl Default for GeoDistanceKmFn {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoDistanceKmFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                ],
                None,
            ),
        }
    }
}

impl Function for GeoDistanceKmFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_number().unwrap();
        let lon1 = args[1].as_number().unwrap();
        let lat2 = args[2].as_number().unwrap();
        let lon2 = args[3].as_number().unwrap();

        let loc1 = Location::new(lat1, lon1);
        let loc2 = Location::new(lat2, lon2);

        let distance = loc1.haversine_distance_to(&loc2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(distance.meters() / 1000.0).unwrap(),
        )))
    }
}

// =============================================================================
// geo_distance_miles(lat1, lon1, lat2, lon2) -> number (miles)
// =============================================================================

pub struct GeoDistanceMilesFn {
    signature: Signature,
}

impl Default for GeoDistanceMilesFn {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoDistanceMilesFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                ],
                None,
            ),
        }
    }
}

impl Function for GeoDistanceMilesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_number().unwrap();
        let lon1 = args[1].as_number().unwrap();
        let lat2 = args[2].as_number().unwrap();
        let lon2 = args[3].as_number().unwrap();

        let loc1 = Location::new(lat1, lon1);
        let loc2 = Location::new(lat2, lon2);

        // 1 meter = 0.000621371 miles
        const METERS_TO_MILES: f64 = 0.000621371;

        let distance = loc1.haversine_distance_to(&loc2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(distance.meters() * METERS_TO_MILES).unwrap(),
        )))
    }
}

// =============================================================================
// geo_bearing(lat1, lon1, lat2, lon2) -> number (degrees 0-360)
// =============================================================================

pub struct GeoBearingFn {
    signature: Signature,
}

impl Default for GeoBearingFn {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoBearingFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                    ArgumentType::Number,
                ],
                None,
            ),
        }
    }
}

impl Function for GeoBearingFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_number().unwrap();
        let lon1 = args[1].as_number().unwrap();
        let lat2 = args[2].as_number().unwrap();
        let lon2 = args[3].as_number().unwrap();

        // Calculate initial bearing using the forward azimuth formula
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let x = delta_lon.sin() * lat2_rad.cos();
        let y = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * delta_lon.cos();

        let bearing_rad = x.atan2(y);
        let mut bearing = bearing_rad.to_degrees();

        // Normalize to 0-360
        if bearing < 0.0 {
            bearing += 360.0;
        }

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(bearing).unwrap(),
        )))
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
    fn test_geo_distance() {
        let runtime = setup();
        // NYC to LA: approximately 3940 km
        let data =
            Variable::from_json(r#"{"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]}"#)
                .unwrap();
        let expr = runtime
            .compile("geo_distance(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let meters = result.as_number().unwrap();
        // Should be approximately 3940000 meters
        assert!(meters > 3900000.0 && meters < 4000000.0);
    }

    #[test]
    fn test_geo_distance_km() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"{"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]}"#)
                .unwrap();
        let expr = runtime
            .compile("geo_distance_km(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let km = result.as_number().unwrap();
        // Should be approximately 3940 km
        assert!(km > 3900.0 && km < 4000.0);
    }

    #[test]
    fn test_geo_distance_miles() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"{"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]}"#)
                .unwrap();
        let expr = runtime
            .compile("geo_distance_miles(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let miles = result.as_number().unwrap();
        // Should be approximately 2450 miles
        assert!(miles > 2400.0 && miles < 2500.0);
    }

    #[test]
    fn test_geo_bearing() {
        let runtime = setup();
        // NYC to LA should be roughly west (270 degrees)
        let data =
            Variable::from_json(r#"{"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]}"#)
                .unwrap();
        let expr = runtime
            .compile("geo_bearing(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let bearing = result.as_number().unwrap();
        // Should be roughly 273 degrees (west-southwest)
        assert!(bearing > 260.0 && bearing < 290.0);
    }

    #[test]
    fn test_geo_distance_same_point() {
        let runtime = setup();
        let data = Variable::from_json(r#"[40.7128, -74.0060]"#).unwrap();
        let expr = runtime
            .compile("geo_distance(@[0], @[1], @[0], @[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let meters = result.as_number().unwrap();
        assert!(meters < 1.0); // Should be essentially 0
    }
}
