"""
Test fixture: Large file with multiple clones
"""


# Duplicate group 1: Logging functions
def log_error(message):
    timestamp = "2024-01-01"
    level = "ERROR"
    print(f"[{timestamp}] {level}: {message}")
    return True


def log_warning(msg):
    timestamp = "2024-01-01"
    level = "ERROR"
    print(f"[{timestamp}] {level}: {msg}")
    return True


# Duplicate group 2: Validation functions
def validate_email(email):
    if "@" not in email:
        return False
    if "." not in email:
        return False
    return True


def validate_username(username):
    if "@" not in username:
        return False
    if "." not in username:
        return False
    return True


# Duplicate group 3: Data processing
def process_data_v1(data):
    results = []
    for item in data:
        if item is not None:
            processed = item * 2
            results.append(processed)
    return results


def process_data_v2(items):
    results = []
    for item in items:
        if item is not None:
            processed = item * 2
            results.append(processed)
    return results


def process_data_v3(values):
    results = []
    for item in values:
        if item is not None:
            processed = item * 2
            results.append(processed)
    return results
