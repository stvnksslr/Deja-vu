"""
Test fixture: Realistic duplicate functions (10-15 lines each)
"""


def calculate_user_score(user_data):
    """Calculate score for a user based on their activity"""
    if not user_data:
        return 0

    base_score = user_data.get("points", 0)
    multiplier = user_data.get("level", 1)
    bonus = user_data.get("achievements", 0) * 10

    total_score = (base_score * multiplier) + bonus

    if total_score < 0:
        total_score = 0

    return total_score


def compute_player_rating(player_info):
    """Compute rating for a player (duplicate logic with different names)"""
    if not player_info:
        return 0

    base_score = player_info.get("points", 0)
    multiplier = player_info.get("level", 1)
    bonus = player_info.get("achievements", 0) * 10

    total_score = (base_score * multiplier) + bonus

    if total_score < 0:
        total_score = 0

    return total_score


def process_order_items(order):
    """Process items in an order and calculate total"""
    if not order or "items" not in order:
        return {"total": 0, "count": 0}

    total_price = 0
    item_count = 0

    for item in order["items"]:
        if item.get("active", True):
            price = item.get("price", 0)
            quantity = item.get("quantity", 1)
            total_price += price * quantity
            item_count += quantity

    return {
        "total": total_price,
        "count": item_count,
        "average": total_price / item_count if item_count > 0 else 0
    }


def calculate_cart_summary(shopping_cart):
    """Calculate summary for shopping cart (duplicate logic)"""
    if not shopping_cart or "items" not in shopping_cart:
        return {"total": 0, "count": 0}

    total_price = 0
    item_count = 0

    for item in shopping_cart["items"]:
        if item.get("active", True):
            price = item.get("price", 0)
            quantity = item.get("quantity", 1)
            total_price += price * quantity
            item_count += quantity

    return {
        "total": total_price,
        "count": item_count,
        "average": total_price / item_count if item_count > 0 else 0
    }
