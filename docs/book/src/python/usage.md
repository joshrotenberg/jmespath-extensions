# Usage

## Basic Searching

The primary function is `search()`, which evaluates a JMESPath expression against data:

```python
import jmespath_extensions as jpx

data = {
    "users": [
        {"name": "alice", "age": 30, "active": True},
        {"name": "bob", "age": 25, "active": False},
        {"name": "carol", "age": 35, "active": True}
    ]
}

# Extract active users
active = jpx.search("users[?active].name", data)
print(active)  # ['alice', 'carol']

# Use extended functions
result = jpx.search("users | sort_by(@, &age) | [*].name", data)
print(result)  # ['bob', 'alice', 'carol']
```

## Compiled Expressions

For repeated use, compile the expression once:

```python
import jmespath_extensions as jpx

# Compile once
expr = jpx.compile_expr("users[?age > `30`].name")

# Use many times
for dataset in datasets:
    result = expr.search(dataset)
    print(result)
```

## Exploring Functions

### List All Functions

```python
import jmespath_extensions as jpx

# All functions
all_funcs = jpx.list_functions()
print(f"Total: {len(all_funcs)} functions")

# Filter by category
string_funcs = jpx.list_functions("string")
print(f"String functions: {string_funcs}")
```

### List Categories

```python
categories = jpx.list_categories()
print(categories)
# ['standard', 'string', 'array', 'object', 'math', 'datetime', ...]
```

### Get Function Details

```python
info = jpx.describe("split")
print(info)
# {
#   'name': 'split',
#   'category': 'String',
#   'signature': 'string, string -> array',
#   'description': 'Split string by delimiter...',
#   'example': 'split(`"a,b,c"`, `","`) -> ["a", "b", "c"]'
# }
```

## Query Libraries

Parse and use `.jpx` query library files:

```python
import jmespath_extensions as jpx

# Parse a query library
content = '''
-- :name active-users
-- :desc Get active users
users[?active]

-- :name user-count
length(users)
'''

library = jpx.parse_query_library(content)

# List queries
for query in library.list():
    print(f"{query.name}: {query.description}")

# Get a specific query
query = library.get("active-users")
if query:
    result = jpx.search(query.expression, data)
    
# Get all query names
names = library.names()
```

### Check if Content is a Query Library

```python
# Detect format
if jpx.is_query_library(content):
    library = jpx.parse_query_library(content)
else:
    # Plain expression
    result = jpx.search(content.strip(), data)
```

## Extended Functions Examples

### String Functions

```python
# Split and join
jpx.search('split(`"a,b,c"`, `","`)', {})  # ['a', 'b', 'c']
jpx.search('join(`"-"`, `["a","b","c"]`)', {})  # 'a-b-c'

# Case conversion
jpx.search('upper(`"hello"`)', {})  # 'HELLO'
jpx.search('snake_case(`"helloWorld"`)', {})  # 'hello_world'
```

### Date/Time Functions

```python
# Current time
jpx.search('now()', {})  # 1699900000 (Unix timestamp)
jpx.search('now_iso()', {})  # '2024-01-15T10:30:00Z'

# Parse and format dates
jpx.search('parse_datetime(`"2024-01-15"`, `"%Y-%m-%d"`)', {})
```

### Math Functions

```python
data = {"values": [1, 2, 3, 4, 5]}

jpx.search('mean(values)', data)  # 3.0
jpx.search('std_dev(values)', data)  # 1.414...
jpx.search('percentile(values, `50`)', data)  # 3
```

### Fuzzy Matching

```python
tools = [
    {"name": "create_user", "description": "Create a new user"},
    {"name": "delete_user", "description": "Delete an existing user"},
    {"name": "list_users", "description": "List all users"}
]

# Fuzzy search
results = jpx.search(
    'fuzzy_search(@, `"name,description"`, `"user create"`)',
    tools
)
```

## Error Handling

```python
import jmespath_extensions as jpx

try:
    # Invalid expression
    jpx.search("invalid[", {})
except ValueError as e:
    print(f"Parse error: {e}")

try:
    # Runtime error
    jpx.search("length(@)", "not an array")
except Exception as e:
    print(f"Evaluation error: {e}")
```
