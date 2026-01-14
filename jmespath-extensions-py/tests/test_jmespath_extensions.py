"""Tests for jmespath-extensions Python bindings."""

import jmespath_extensions as jpx
import pytest


class TestSearch:
    """Tests for the search function."""

    def test_upper(self):
        result = jpx.search("upper(name)", {"name": "alice"})
        assert result == "ALICE"

    def test_lower(self):
        result = jpx.search("lower(name)", {"name": "ALICE"})
        assert result == "alice"

    def test_sum(self):
        result = jpx.search("sum(values)", {"values": [1, 2, 3, 4, 5]})
        assert result == 15

    def test_unique(self):
        result = jpx.search("unique(items)", {"items": [1, 2, 2, 3, 3, 3]})
        assert result == [1, 2, 3]

    def test_first(self):
        result = jpx.search("first(items)", {"items": [1, 2, 3]})
        assert result == 1

    def test_last(self):
        result = jpx.search("last(items)", {"items": [1, 2, 3]})
        assert result == 3

    def test_nested_access(self):
        data = {"user": {"profile": {"name": "alice"}}}
        result = jpx.search("user.profile.name", data)
        assert result == "alice"

    def test_filter_expression(self):
        data = [{"age": 25}, {"age": 17}, {"age": 30}]
        result = jpx.search("[?age > `18`]", data)
        assert len(result) == 2

    def test_null_result(self):
        result = jpx.search("missing", {"name": "alice"})
        assert result is None

    def test_boolean_result(self):
        result = jpx.search("contains(items, `2`)", {"items": [1, 2, 3]})
        assert result is True

    def test_invalid_expression(self):
        with pytest.raises(ValueError, match="Invalid JMESPath"):
            jpx.search("[invalid", {"name": "alice"})


class TestCompile:
    """Tests for the compile function."""

    def test_compile_and_search(self):
        expr = jpx.compile("users[*].name")
        result = expr.search({"users": [{"name": "alice"}, {"name": "bob"}]})
        assert result == ["alice", "bob"]

    def test_compiled_reuse(self):
        expr = jpx.compile("sum(values)")
        assert expr.search({"values": [1, 2, 3]}) == 6
        assert expr.search({"values": [10, 20, 30]}) == 60

    def test_compile_invalid(self):
        with pytest.raises(ValueError, match="Invalid JMESPath"):
            jpx.compile("[invalid")

    def test_repr(self):
        expr = jpx.compile("name")
        assert "name" in repr(expr)

    def test_str(self):
        expr = jpx.compile("users[*].name")
        assert str(expr) == "users[*].name"


class TestListFunctions:
    """Tests for the list_functions function."""

    def test_list_all(self):
        funcs = jpx.list_functions()
        assert len(funcs) > 300

    def test_list_by_category(self):
        funcs = jpx.list_functions("string")
        assert len(funcs) > 0
        # Check some expected string functions exist
        assert any(f in funcs for f in ["upper", "lower", "trim"])

    def test_list_math(self):
        funcs = jpx.list_functions("math")
        assert len(funcs) > 0
        assert any(f in funcs for f in ["sum", "avg", "sqrt"])


class TestListCategories:
    """Tests for the list_categories function."""

    def test_categories(self):
        cats = jpx.list_categories()
        assert "string" in cats
        assert "math" in cats
        assert "array" in cats
        assert "object" in cats
        assert "datetime" in cats


class TestDescribe:
    """Tests for the describe function."""

    def test_describe_upper(self):
        info = jpx.describe("upper")
        assert info is not None
        assert info["name"] == "upper"
        assert info["category"] == "string"
        assert "description" in info
        assert "signature" in info
        assert "example" in info
        assert info["is_standard"] is False

    def test_describe_standard_function(self):
        info = jpx.describe("length")
        assert info is not None
        assert info["is_standard"] is True

    def test_describe_nonexistent(self):
        info = jpx.describe("nonexistent_function_xyz")
        assert info is None


class TestStandardFunctions:
    """Tests for standard JMESPath functions."""

    def test_length(self):
        assert jpx.search("length(items)", {"items": [1, 2, 3]}) == 3

    def test_sort(self):
        assert jpx.search("sort(items)", {"items": [3, 1, 2]}) == [1, 2, 3]

    def test_reverse(self):
        assert jpx.search("reverse(items)", {"items": [1, 2, 3]}) == [3, 2, 1]

    def test_keys(self):
        result = jpx.search("keys(@)", {"a": 1, "b": 2})
        assert set(result) == {"a", "b"}

    def test_values(self):
        result = jpx.search("values(@)", {"a": 1, "b": 2})
        assert set(result) == {1, 2}

    def test_join(self):
        result = jpx.search("join(', ', items)", {"items": ["a", "b", "c"]})
        assert result == "a, b, c"


class TestExtensionFunctions:
    """Tests for extension functions."""

    def test_chunk(self):
        result = jpx.search("chunk(items, `2`)", {"items": [1, 2, 3, 4, 5, 6]})
        assert result == [[1, 2], [3, 4], [5, 6]]

    def test_pick(self):
        data = {"name": "alice", "email": "a@b.com", "age": 30}
        result = jpx.search("pick(@, ['name', 'email'])", data)
        assert result == {"name": "alice", "email": "a@b.com"}

    def test_omit(self):
        data = {"name": "alice", "email": "a@b.com", "age": 30}
        result = jpx.search("omit(@, ['age'])", data)
        assert result == {"name": "alice", "email": "a@b.com"}

    def test_median(self):
        result = jpx.search("median(values)", {"values": [1, 2, 3, 4, 5]})
        assert result == 3

    def test_camel_case(self):
        result = jpx.search("camel_case(name)", {"name": "hello_world"})
        assert result == "helloWorld"

    def test_snake_case(self):
        result = jpx.search("snake_case(name)", {"name": "helloWorld"})
        assert result == "hello_world"


class TestVersion:
    """Tests for version info."""

    def test_version_exists(self):
        assert hasattr(jpx, "__version__")
        assert isinstance(jpx.__version__, str)
        assert len(jpx.__version__) > 0
