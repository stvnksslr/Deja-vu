"""Test file for AST-based clone detection - Type-3 clones"""


def calculate_discount(price, discount_rate):
    """Calculate discount amount"""
    if price < 0:
        raise ValueError("Price cannot be negative")

    discount = price * discount_rate
    print(f"Calculating discount: {discount}")

    return discount


def calculate_tax(amount, tax_rate):
    """Calculate tax amount"""
    if amount < 0:
        raise ValueError("Amount cannot be negative")

    # Similar logic but missing the print statement
    tax = amount * tax_rate

    return tax


def calculate_shipping(weight, rate):
    """Calculate shipping cost"""
    if weight < 0:
        raise ValueError("Weight cannot be negative")

    cost = weight * rate
    print(f"Calculating shipping: {cost}")
    # Extra validation
    if cost > 100:
        print("Warning: High shipping cost")

    return cost


class PriceCalculator:
    def __init__(self, base_price):
        self.base_price = base_price

    def apply_discount(self, rate):
        if rate < 0 or rate > 1:
            raise ValueError("Rate must be between 0 and 1")
        discounted = self.base_price * (1 - rate)
        return discounted

    def apply_tax(self, rate):
        if rate < 0 or rate > 1:
            raise ValueError("Rate must be between 0 and 1")
        # Similar logic to apply_discount
        taxed = self.base_price * (1 + rate)
        return taxed


class TaxCalculator:
    def __init__(self, base_amount):
        self.base_amount = base_amount

    def apply_sales_tax(self, rate):
        if rate < 0 or rate > 1:
            raise ValueError("Rate must be between 0 and 1")
        # Very similar to PriceCalculator.apply_tax
        taxed = self.base_amount * (1 + rate)
        return taxed

    def apply_discount(self, rate):
        if rate < 0 or rate > 1:
            raise ValueError("Rate must be between 0 and 1")
        discounted = self.base_amount * (1 - rate)
        return discounted
