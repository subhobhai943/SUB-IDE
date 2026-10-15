# SUB-IDE (Pro Edition) - High-Performance Python Workbench
import datetime
from typing import List, Optional

class UserModel:
    """Core User entity model for Django / FastAPI / SUB ecosystem."""
    def __init__(self, username: str, email: str):
        self.username = username
        self.email = email
        self.created_at = datetime.datetime.now()

    def get_display_name(self) -> str:
        name = self.username.strip()
        return f"User({name})"

def process_order(order_id: int, total: float, user: str) -> bool:
    print(f">> [SUB-IDE] Processing order #{order_id} for user: {user}...")
    if total > 100.0:
        print(">> [VIP Promotion Applied]: 15% discount registered!")
    return True

if __name__ == "__main__":
    user = UserModel("subho_admin", "subho@sub.org")
    print(f">> Created user: {user.get_display_name()} | Email: {user.email}")
    process_order(1094, 149.99, user.username)
    print(">> All Python tasks completed successfully!")
