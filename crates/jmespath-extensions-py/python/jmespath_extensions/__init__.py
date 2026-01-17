"""
jmespath-extensions - JMESPath with 400+ extension functions.

This package provides a JMESPath implementation with over 400 additional functions
for strings, arrays, dates, hashing, encoding, and more.

Example:
    >>> import jmespath_extensions as jpx
    >>> jpx.search("upper(name)", {"name": "alice"})
    'ALICE'

    >>> jpx.search("sum(values)", {"values": [1, 2, 3, 4, 5]})
    15

    >>> jpx.search("format_date(now(), '%Y-%m-%d')", {})
    '2024-01-15'

For a complete list of functions:
    >>> jpx.list_categories()
    ['standard', 'string', 'array', 'object', 'math', ...]

    >>> jpx.list_functions("string")
    ['upper', 'lower', 'trim', 'split', ...]

    >>> jpx.describe("upper")
    {'name': 'upper', 'description': 'Convert string to uppercase', ...}
"""

from jmespath_extensions._core import (
    CompiledExpression,
    __version__,
    compile,
    describe,
    list_categories,
    list_functions,
    search,
)

__all__ = [
    "search",
    "compile",
    "list_functions",
    "list_categories",
    "describe",
    "CompiledExpression",
    "__version__",
]
