"""
Utility functions with more duplicates
"""


def process_list_method1(items):
    """Process a list of items"""
    results = []
    for item in items:
        if item is not None:
            processed = item * 2
            results.append(processed)
    return results


def process_list_method2(elements):
    """Process a list of elements (Type-2 clone)"""
    results = []
    for element in elements:
        if element is not None:
            processed = element * 2
            results.append(processed)
    return results


def filter_positive_numbers(numbers):
    """Filter out negative numbers"""
    filtered = []
    for num in numbers:
        if num > 0:
            filtered.append(num)
        else:
            pass
    return filtered


def filter_negative_numbers(values):
    """Filter out positive numbers"""
    filtered = []
    for val in values:
        if val < 0:
            filtered.append(val)
        else:
            pass
    return filtered


# Exact duplicate class methods (Type-1 clones)
class DataProcessorA:
    def __init__(self, name):
        self.name = name
        self.data = []

    def add_data(self, item):
        """Add an item to data"""
        if item is not None:
            self.data.append(item)
            print(f"Added {item} to {self.name}")
            return True
        return False

    def get_count(self):
        """Get count of items"""
        return len(self.data)


class DataProcessorB:
    def __init__(self, name):
        self.name = name
        self.data = []

    def add_data(self, item):
        """Add an item to data (duplicate method!)"""
        if item is not None:
            self.data.append(item)
            print(f"Added {item} to {self.name}")
            return True
        return False

    def get_count(self):
        """Get count of items (duplicate method!)"""
        return len(self.data)


# String formatting duplicates
def format_user_info_v1(name, age, city):
    """Format user information"""
    info = f"Name: {name}\n"
    info += f"Age: {age}\n"
    info += f"City: {city}\n"
    info += "-" * 40 + "\n"
    return info


def format_user_info_v2(name, age, city):
    """Format user information (exact duplicate!)"""
    info = f"Name: {name}\n"
    info += f"Age: {age}\n"
    info += f"City: {city}\n"
    info += "-" * 40 + "\n"
    return info


def format_product_info(title, price, category):
    """Format product information"""
    info = f"Title: {title}\n"
    info += f"Price: {price}\n"
    info += f"Category: {category}\n"
    info += "-" * 40 + "\n"
    return info
