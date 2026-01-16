# API Reference

## Functions

### search

```python
def search(expression: str, data: Any) -> Any
```

Evaluate a JMESPath expression against data.

**Parameters:**
- `expression`: JMESPath expression string
- `data`: Python data (dict, list, str, int, float, bool, None)

**Returns:** The result of the expression evaluation

**Raises:** `ValueError` for parse errors, `Exception` for runtime errors

**Example:**
```python
result = jpx.search("users[0].name", {"users": [{"name": "alice"}]})
# Returns: "alice"
```

---

### compile_expr

```python
def compile_expr(expression: str) -> CompiledExpression
```

Compile a JMESPath expression for repeated use.

**Parameters:**
- `expression`: JMESPath expression string

**Returns:** `CompiledExpression` object

**Raises:** `ValueError` for parse errors

**Example:**
```python
expr = jpx.compile_expr("length(@)")
result = expr.search([1, 2, 3])  # 3
```

---

### list_functions

```python
def list_functions(category: Optional[str] = None) -> List[str]
```

List available function names.

**Parameters:**
- `category`: Optional category filter (e.g., "string", "math", "datetime")

**Returns:** List of function names

**Example:**
```python
all_funcs = jpx.list_functions()
string_funcs = jpx.list_functions("string")
```

---

### list_categories

```python
def list_categories() -> List[str]
```

List all function categories.

**Returns:** List of category names

**Example:**
```python
categories = jpx.list_categories()
# ['standard', 'string', 'array', 'object', 'math', ...]
```

---

### describe

```python
def describe(name: str) -> Optional[Dict]
```

Get detailed information about a function.

**Parameters:**
- `name`: Function name or alias

**Returns:** Dictionary with function info, or `None` if not found

**Return fields:**
- `name`: Function name
- `category`: Category name
- `signature`: Type signature
- `description`: Description text
- `example`: Example usage

**Example:**
```python
info = jpx.describe("split")
print(info["signature"])  # "string, string -> array"
```

---

### parse_query_library

```python
def parse_query_library(content: str) -> QueryLibrary
```

Parse a query library from string content.

**Parameters:**
- `content`: Query library content in `.jpx` format

**Returns:** `QueryLibrary` object

**Raises:** `ValueError` for parse errors

**Example:**
```python
library = jpx.parse_query_library('''
-- :name my-query
-- :desc A sample query
users[?active]
''')
```

---

### is_query_library

```python
def is_query_library(content: str) -> bool
```

Check if content appears to be a query library format.

**Parameters:**
- `content`: String content to check

**Returns:** `True` if content starts with query library directives

**Example:**
```python
if jpx.is_query_library(content):
    library = jpx.parse_query_library(content)
```

---

## Classes

### CompiledExpression

A compiled JMESPath expression for efficient repeated evaluation.

#### Methods

##### search

```python
def search(self, data: Any) -> Any
```

Evaluate this expression against data.

**Parameters:**
- `data`: Python data to evaluate against

**Returns:** Evaluation result

---

### QueryLibrary

A collection of named queries parsed from a `.jpx` file.

#### Methods

##### get

```python
def get(self, name: str) -> Optional[NamedQuery]
```

Get a query by name.

**Parameters:**
- `name`: Query name

**Returns:** `NamedQuery` or `None`

##### list

```python
def list(self) -> List[NamedQuery]
```

Get all queries in the library.

**Returns:** List of `NamedQuery` objects

##### names

```python
def names(self) -> List[str]
```

Get all query names.

**Returns:** List of query names

---

### NamedQuery

A single named query from a query library.

#### Properties

- `name: str` - Query name
- `description: Optional[str]` - Query description
- `expression: str` - JMESPath expression
- `line_number: int` - Line number in source file

**Example:**
```python
query = library.get("my-query")
if query:
    print(f"{query.name}: {query.description}")
    result = jpx.search(query.expression, data)
```
