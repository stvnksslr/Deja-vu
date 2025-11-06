"""Module docstring should be ignored"""


def add_v1(a, b):
    """First docstring about adding"""
    result = a + b
    return result


def add_v2(a, b):
    """Second completely different docstring"""
    result = a + b
    return result


# These functions should NOT be duplicates (different code, same docstring)
def multiply(x, y):
    """Do math operation"""
    return x * y


def subtract(x, y):
    """Do math operation"""
    return x - y
