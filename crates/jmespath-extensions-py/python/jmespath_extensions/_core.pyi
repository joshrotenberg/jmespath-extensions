"""Type stubs for jmespath_extensions._core"""

from typing import Any

__version__: str

class CompiledExpression:
    """A compiled JMESPath expression for efficient repeated searches."""

    def search(self, data: Any) -> Any:
        """
        Search JSON data using this compiled expression.

        Args:
            data: JSON-compatible Python data (dict, list, str, int, float, bool, None)

        Returns:
            The result of evaluating the expression against the data

        Raises:
            ValueError: If evaluation fails
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

def search(expression: str, data: Any) -> Any:
    """
    Search JSON data using a JMESPath expression with extended functions.

    Args:
        expression: A JMESPath expression string
        data: JSON-compatible Python data (dict, list, str, int, float, bool, None)

    Returns:
        The result of evaluating the expression against the data

    Raises:
        ValueError: If the expression is invalid or evaluation fails

    Example:
        >>> import jmespath_extensions as jpx
        >>> jpx.search("upper(name)", {"name": "alice"})
        'ALICE'
        >>> jpx.search("sum(values)", {"values": [1, 2, 3]})
        6
    """
    ...

def compile(expression: str) -> CompiledExpression:
    """
    Compile a JMESPath expression for repeated use.

    Args:
        expression: A JMESPath expression string

    Returns:
        A compiled Expression object that can be reused

    Raises:
        ValueError: If the expression is invalid

    Example:
        >>> import jmespath_extensions as jpx
        >>> expr = jpx.compile("users[*].name")
        >>> expr.search({"users": [{"name": "alice"}, {"name": "bob"}]})
        ['alice', 'bob']
    """
    ...

def list_functions(category: str | None = None) -> list[str]:
    """
    List all available extension functions.

    Args:
        category: Optional category to filter by (e.g., "string", "math", "datetime")

    Returns:
        A list of function names

    Example:
        >>> import jmespath_extensions as jpx
        >>> "upper" in jpx.list_functions("string")
        True
        >>> len(jpx.list_functions()) > 300
        True
    """
    ...

def list_categories() -> list[str]:
    """
    List all available function categories.

    Returns:
        A list of category names

    Example:
        >>> import jmespath_extensions as jpx
        >>> categories = jpx.list_categories()
        >>> "string" in categories
        True
        >>> "math" in categories
        True
    """
    ...

def describe(name: str) -> dict[str, Any] | None:
    """
    Get information about a specific function.

    Args:
        name: The function name

    Returns:
        A dictionary with function info, or None if not found.
        Keys: name, category, description, signature, example, is_standard

    Example:
        >>> import jmespath_extensions as jpx
        >>> info = jpx.describe("upper")
        >>> info["description"]
        'Convert string to uppercase'
        >>> info["is_standard"]
        False
    """
    ...
