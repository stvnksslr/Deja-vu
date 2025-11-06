"""
Example calculator with duplicate code (Type-1 and Type-2 clones)
"""


def add_numbers(a, b):
    """Add two numbers"""
    result = a + b
    print(f"Adding {a} and {b}")
    print(f"Result is {result}")
    return result


def subtract_numbers(x, y):
    """Subtract two numbers"""
    result = x - y
    print(f"Subtracting {x} and {y}")
    print(f"Result is {result}")
    return result


def multiply_numbers(num1, num2):
    """Multiply two numbers"""
    result = num1 * num2
    print(f"Multiplying {num1} and {num2}")
    print(f"Result is {result}")
    return result


def divide_numbers(dividend, divisor):
    """Divide two numbers"""
    if divisor == 0:
        print("Error: Division by zero!")
        return None
    result = dividend / divisor
    print(f"Dividing {dividend} by {divisor}")
    print(f"Result is {result}")
    return result


def power_numbers(base, exponent):
    """Raise base to exponent"""
    result = base ** exponent
    print(f"Raising {base} to power {exponent}")
    print(f"Result is {result}")
    return result


# Duplicate logging functions (Type-1 clones)
def log_operation_v1(operation, x, y, result):
    """Log an operation - version 1"""
    timestamp = "2024-01-01"
    message = f"[{timestamp}] Operation: {operation}"
    print(message)
    print(f"  Input: {x}, {y}")
    print(f"  Output: {result}")
    print("  Status: Success")


def log_operation_v2(operation, x, y, result):
    """Log an operation - version 2 (exact duplicate!)"""
    timestamp = "2024-01-01"
    message = f"[{timestamp}] Operation: {operation}"
    print(message)
    print(f"  Input: {x}, {y}")
    print(f"  Output: {result}")
    print("  Status: Success")


# Type-2 clones (renamed variables)
def validate_input_a(value, min_val, max_val):
    """Validate input is in range"""
    if value < min_val:
        return False
    if value > max_val:
        return False
    return True


def validate_input_b(num, lower, upper):
    """Validate input is in range (renamed variables)"""
    if num < lower:
        return False
    if num > upper:
        return False
    return True
