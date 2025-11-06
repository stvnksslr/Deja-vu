#!/usr/bin/env python3
"""Test file with intentional duplicates"""

def calculate_sum(a, b):
    """Calculate sum of two numbers"""
    result = a + b
    return result

def calculate_total(x, y):
    """Calculate total of two values"""
    total = x + y
    return total

class DataProcessor:
    def process_data(self, data):
        """Process the data"""
        filtered = [x for x in data if x > 0]
        result = sum(filtered)
        return result

class DataHandler:
    def handle_data(self, data):
        """Handle the data"""
        filtered = [x for x in data if x > 0]
        result = sum(filtered)
        return result

def another_duplicate():
    """Another duplicate function"""
    items = []
    for i in range(10):
        items.append(i * 2)
    return items

def similar_duplicate():
    """Similar duplicate function"""
    items = []
    for i in range(10):
        items.append(i * 2)
    return items
