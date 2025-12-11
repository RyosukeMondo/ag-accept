
import uiautomation as auto
import inspect

print("uiautomation version:", getattr(auto, "VERSION", "Unknown"))
print("PaneControl attributes:")
try:
    c = auto.PaneControl()
    print("Has FindFirst:", hasattr(c, "FindFirst"))
    print("Has find_first:", hasattr(c, "find_first"))
    
    # List all finding related methods
    for name in dir(c):
        if "Find" in name:
            print(f"- {name}")
except Exception as e:
    print(f"Error inspecting PaneControl: {e}")

print("\nMethod resolution order for PaneControl:")
print(auto.PaneControl.mro())
