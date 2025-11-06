"""Test file to verify docstrings are ignored"""


def calculate_sum_v1(numbers):
    """This function calculates the sum of a list of numbers.
    It iterates through each number and adds them together.
    Returns the total sum as an integer or float."""
    total = 0
    for num in numbers:
        total = total + num
    return total


def calculate_sum_v2(numbers):
    """Completely different docstring here!
    This one talks about addition instead."""
    total = 0
    for num in numbers:
        total = total + num
    return total


def process_data_v1(items):
    """Process a list of items by filtering and transforming them."""
    result = []
    for item in items:
        if item is not None:
            processed = item * 2
            result.append(processed)
    return result


def process_data_v2(items):
    """Different description: transforms input data."""
    result = []
    for item in items:
        if item is not None:
            processed = item * 2
            result.append(processed)
    return result
