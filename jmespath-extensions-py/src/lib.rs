//! Python bindings for jmespath-extensions.
//!
//! This module provides Python access to the extended JMESPath functions
//! implemented in the jmespath_extensions Rust crate.

use std::sync::OnceLock;

use jmespath::{Runtime, Variable};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Value;

/// Global runtime with all extensions registered.
/// Using OnceLock for thread-safe lazy initialization.
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        jmespath_extensions::register_all(&mut runtime);
        runtime
    })
}

/// Search JSON data using a JMESPath expression with extended functions.
///
/// Args:
///     expression: A JMESPath expression string
///     data: JSON data as a Python object (dict, list, str, int, float, bool, None)
///
/// Returns:
///     The result of evaluating the expression against the data
///
/// Raises:
///     ValueError: If the expression is invalid or evaluation fails
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> jpx.search("upper(name)", {"name": "alice"})
///     'ALICE'
#[pyfunction]
#[pyo3(signature = (expression, data))]
fn search(expression: &str, data: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let runtime = get_runtime();

    // Compile the expression
    let expr = runtime
        .compile(expression)
        .map_err(|e| PyValueError::new_err(format!("Invalid JMESPath expression: {}", e)))?;

    // Convert Python object to serde_json::Value
    let json_value = python_to_json(data)?;

    // Convert to jmespath Variable
    let variable: Variable = serde_json::from_value(json_value)
        .map_err(|e| PyValueError::new_err(format!("Failed to convert data: {}", e)))?;

    // Execute the search
    let result = expr
        .search(&variable)
        .map_err(|e| PyValueError::new_err(format!("JMESPath evaluation error: {}", e)))?;

    // Convert result back to Python
    let result_json: Value = serde_json::to_value(&*result)
        .map_err(|e| PyValueError::new_err(format!("Failed to convert result: {}", e)))?;

    Python::with_gil(|py| json_to_python(py, &result_json))
}

/// Compile a JMESPath expression for repeated use.
///
/// Args:
///     expression: A JMESPath expression string
///
/// Returns:
///     A compiled Expression object
///
/// Raises:
///     ValueError: If the expression is invalid
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> expr = jpx.compile("users[*].name | upper(@)")
///     >>> expr.search({"users": [{"name": "alice"}, {"name": "bob"}]})
///     ['ALICE', 'BOB']
#[pyfunction]
#[pyo3(name = "compile")]
fn compile_expr(expression: &str) -> PyResult<CompiledExpression> {
    let runtime = get_runtime();

    // Validate expression by compiling it
    runtime
        .compile(expression)
        .map_err(|e| PyValueError::new_err(format!("Invalid JMESPath expression: {}", e)))?;

    Ok(CompiledExpression {
        expression: expression.to_string(),
    })
}

/// A compiled JMESPath expression for efficient repeated searches.
#[pyclass]
#[derive(Clone)]
struct CompiledExpression {
    expression: String,
}

#[pymethods]
impl CompiledExpression {
    /// Search JSON data using this compiled expression.
    ///
    /// Args:
    ///     data: JSON data as a Python object
    ///
    /// Returns:
    ///     The result of evaluating the expression against the data
    fn search(&self, data: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        search(&self.expression, data)
    }

    fn __repr__(&self) -> String {
        format!("CompiledExpression({:?})", self.expression)
    }

    fn __str__(&self) -> String {
        self.expression.clone()
    }
}

/// List all available extension functions.
///
/// Args:
///     category: Optional category to filter by (e.g., "string", "math", "datetime")
///
/// Returns:
///     A list of function names
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> jpx.list_functions("string")[:5]
///     ['upper', 'lower', 'trim', 'split', 'replace']
#[pyfunction]
#[pyo3(signature = (category=None))]
fn list_functions(category: Option<&str>) -> PyResult<Vec<String>> {
    use jmespath_extensions::registry::FunctionRegistry;

    let mut registry = FunctionRegistry::new();
    registry.register_all();

    let functions: Vec<String> = registry
        .functions()
        .filter(|f| {
            if let Some(cat) = category {
                let cat_lower = cat.to_lowercase();
                let func_cat = format!("{:?}", f.category).to_lowercase();
                func_cat == cat_lower
            } else {
                true
            }
        })
        .map(|f| f.name.to_string())
        .collect();

    Ok(functions)
}

/// List all available function categories.
///
/// Returns:
///     A list of category names
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> "string" in jpx.list_categories()
///     True
#[pyfunction]
fn list_categories() -> Vec<String> {
    use jmespath_extensions::registry::Category;

    Category::all()
        .iter()
        .map(|c| format!("{:?}", c).to_lowercase())
        .collect()
}

/// Get information about a specific function.
///
/// Args:
///     name: The function name
///
/// Returns:
///     A dictionary with function info (name, category, description, signature, example)
///     or None if not found
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> info = jpx.describe("upper")
///     >>> info["description"]
///     'Convert string to uppercase'
#[pyfunction]
fn describe(py: Python<'_>, name: &str) -> PyResult<Option<PyObject>> {
    use jmespath_extensions::registry::FunctionRegistry;

    let mut registry = FunctionRegistry::new();
    registry.register_all();

    if let Some(info) = registry.functions().find(|f| f.name == name) {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("name", info.name)?;
        dict.set_item("category", format!("{:?}", info.category).to_lowercase())?;
        dict.set_item("description", info.description)?;
        dict.set_item("signature", info.signature)?;
        dict.set_item("example", info.example)?;
        dict.set_item("is_standard", info.is_standard)?;
        Ok(Some(dict.into()))
    } else {
        Ok(None)
    }
}

/// Convert a Python object to serde_json::Value
fn python_to_json(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(list) = obj.downcast::<pyo3::types::PyList>() {
        let arr: Result<Vec<Value>, _> = list.iter().map(|item| python_to_json(&item)).collect();
        Ok(Value::Array(arr?))
    } else if let Ok(dict) = obj.downcast::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = key
                .extract::<String>()
                .map_err(|_| PyValueError::new_err("Dictionary keys must be strings"))?;
            map.insert(key_str, python_to_json(&value)?);
        }
        Ok(Value::Object(map))
    } else {
        Err(PyValueError::new_err(format!(
            "Cannot convert {} to JSON",
            obj.get_type().name()?
        )))
    }
}

/// Convert serde_json::Value to a Python object
fn json_to_python(py: Python<'_>, value: &Value) -> PyResult<PyObject> {
    use pyo3::IntoPyObject;

    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok((*b).into_pyobject(py)?.to_owned().into_any().unbind()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        Value::String(s) => Ok(s.as_str().into_pyobject(py)?.to_owned().into_any().unbind()),
        Value::Array(arr) => {
            let list = pyo3::types::PyList::empty(py);
            for item in arr {
                list.append(json_to_python(py, item)?)?;
            }
            Ok(list.into())
        }
        Value::Object(obj) => {
            let dict = pyo3::types::PyDict::new(py);
            for (key, val) in obj {
                dict.set_item(key, json_to_python(py, val)?)?;
            }
            Ok(dict.into())
        }
    }
}

// ============================================================================
// Query Library Support
// ============================================================================

/// A named query from a query library.
#[pyclass]
#[derive(Clone)]
struct NamedQuery {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    description: Option<String>,
    #[pyo3(get)]
    expression: String,
    #[pyo3(get)]
    line_number: usize,
}

#[pymethods]
impl NamedQuery {
    fn __repr__(&self) -> String {
        match &self.description {
            Some(desc) => format!("NamedQuery(name={:?}, description={:?})", self.name, desc),
            None => format!("NamedQuery(name={:?})", self.name),
        }
    }

    fn __str__(&self) -> String {
        self.expression.clone()
    }
}

/// A collection of named queries parsed from a .jpx file.
#[pyclass]
#[derive(Clone)]
struct QueryLibrary {
    inner: jmespath_extensions::query_library::QueryLibrary,
}

#[pymethods]
impl QueryLibrary {
    /// Get a query by name.
    ///
    /// Args:
    ///     name: The query name
    ///
    /// Returns:
    ///     The NamedQuery if found, None otherwise
    fn get(&self, name: &str) -> Option<NamedQuery> {
        self.inner.get(name).map(|q| NamedQuery {
            name: q.name.clone(),
            description: q.description.clone(),
            expression: q.expression.clone(),
            line_number: q.line_number,
        })
    }

    /// Get all query names.
    ///
    /// Returns:
    ///     A list of query names
    fn names(&self) -> Vec<String> {
        self.inner.names().into_iter().map(String::from).collect()
    }

    /// Get all queries.
    ///
    /// Returns:
    ///     A list of NamedQuery objects
    fn list(&self) -> Vec<NamedQuery> {
        self.inner
            .list()
            .iter()
            .map(|q| NamedQuery {
                name: q.name.clone(),
                description: q.description.clone(),
                expression: q.expression.clone(),
                line_number: q.line_number,
            })
            .collect()
    }

    /// Get the number of queries in the library.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Check if a query exists by name.
    fn __contains__(&self, name: &str) -> bool {
        self.inner.get(name).is_some()
    }

    /// Get a query by name (dict-like access).
    fn __getitem__(&self, name: &str) -> PyResult<NamedQuery> {
        self.get(name)
            .ok_or_else(|| PyValueError::new_err(format!("Query '{}' not found", name)))
    }

    fn __repr__(&self) -> String {
        let names = self.inner.names().join(", ");
        format!("QueryLibrary([{}])", names)
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<QueryLibraryIter> {
        Ok(QueryLibraryIter {
            queries: slf.list(),
            index: 0,
        })
    }
}

#[pyclass]
struct QueryLibraryIter {
    queries: Vec<NamedQuery>,
    index: usize,
}

#[pymethods]
impl QueryLibraryIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<NamedQuery> {
        if slf.index < slf.queries.len() {
            let query = slf.queries[slf.index].clone();
            slf.index += 1;
            Some(query)
        } else {
            None
        }
    }
}

/// Parse a query library from content.
///
/// Query libraries use the .jpx format with named queries:
///
/// ```text
/// -- :name greet
/// -- :desc Simple greeting
/// `"hello"`
///
/// -- :name count
/// length(@)
/// ```
///
/// Args:
///     content: The query library content as a string
///
/// Returns:
///     A QueryLibrary object
///
/// Raises:
///     ValueError: If the content is invalid
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> lib = jpx.parse_query_library('''
///     ... -- :name count
///     ... length(@)
///     ...
///     ... -- :name first
///     ... @[0]
///     ... ''')
///     >>> lib.names()
///     ['count', 'first']
///     >>> lib['count'].expression
///     'length(@)'
#[pyfunction]
fn parse_query_library(content: &str) -> PyResult<QueryLibrary> {
    let inner = jmespath_extensions::query_library::QueryLibrary::parse(content)
        .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;

    Ok(QueryLibrary { inner })
}

/// Check if content looks like a query library.
///
/// Args:
///     content: The content to check
///
/// Returns:
///     True if the content appears to be a query library (starts with -- :name)
///
/// Example:
///     >>> import jmespath_extensions as jpx
///     >>> jpx.is_query_library("-- :name foo\\nlength(@)")
///     True
///     >>> jpx.is_query_library("length(@)")
///     False
#[pyfunction]
fn is_query_library(content: &str) -> bool {
    jmespath_extensions::query_library::is_query_library(content)
}

/// jmespath_extensions Python module
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(compile_expr, m)?)?;
    m.add_function(wrap_pyfunction!(list_functions, m)?)?;
    m.add_function(wrap_pyfunction!(list_categories, m)?)?;
    m.add_function(wrap_pyfunction!(describe, m)?)?;
    m.add_function(wrap_pyfunction!(parse_query_library, m)?)?;
    m.add_function(wrap_pyfunction!(is_query_library, m)?)?;
    m.add_class::<CompiledExpression>()?;
    m.add_class::<QueryLibrary>()?;
    m.add_class::<NamedQuery>()?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
