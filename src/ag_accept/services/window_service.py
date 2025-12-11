import uiautomation as auto
from typing import List, Optional, Any
import time

class WindowService:
    """
    Service for managing windows, including finding, focusing, and structure analysis.
    """
    def __init__(self):
        self.previous_focus_control = None

    def get_root_control(self) -> Any:
        return auto.GetRootControl()

    def get_all_windows(self, exclude_titles: List[str] = []) -> List[Any]:
        """
        Returns a listing of all top-level windows, optionally excluding some by title.
        """
        windows = []
        try:
            root = self.get_root_control()
            for window in root.GetChildren():
                name = window.Name
                # Simple exclusion check
                if any(ex.lower() in name.lower() for ex in exclude_titles if ex):
                    continue
                windows.append(window)
        except Exception:
            pass
        return windows

    def find_window_by_title(self, title_part: str, exclude_titles: List[str] = []) -> Optional[Any]:
        """
        Finds a window that contains 'title_part' in its name.
        Returns the best match or None.
        """
        try:
            root = self.get_root_control()
            best_match = None
            
            for window in root.GetChildren():
                name = window.Name
                
                # Check exclusions
                if any(ex.lower() in name.lower() for ex in exclude_titles if ex):
                    continue

                # Exact match priority
                if name == title_part:
                    return window
                
                # Partial match
                if title_part in name:
                    if not best_match:
                        best_match = window
            
            return best_match
        except Exception:
            return None

    def focus_window(self, window: Any) -> None:
        """
        Focuses the specified window and saves the currently focused element
        to enable 'restore_previous_focus'.
        """
        try:
            # Save current focus
            self.previous_focus_control = auto.GetFocusedControl()
        except:
            self.previous_focus_control = None

        try:
            window.SetFocus()
        except Exception as e:
            # Log or re-raise? For service, maybe just warn or let caller handle.
            # We'll assume caller handles specific errors if needed.
            print(f"WindowService: Failed to focus window: {e}")

    def restore_previous_focus(self) -> None:
        """
        Restores focus to the element that was focused before 'focus_window' was last called.
        """
        if self.previous_focus_control:
            try:
                self.previous_focus_control.SetFocus()
            except Exception as e:
                print(f"WindowService: Failed to restore focus: {e}")
            finally:
                self.previous_focus_control = None

    def get_window_structure(self, window: Any, depth: int = 0, max_depth: int = 5) -> str:
        """
        Returns a string representation of the window structure for debugging.
        """
        if depth > max_depth:
            return ""
        
        indent = "  " * depth
        try:
            name = window.Name
            control_type = window.ControlTypeName
            
            extra = ""
            try:
                rect = window.BoundingRectangle
                extra += f" Rect:{rect}"
            except: pass
            
            try:
                auto_id = window.AutomationId
                if auto_id:
                    extra += f" AutoID:{auto_id}"
            except: pass
            
            info = f"\n{indent}- [{control_type}] '{name}'{extra}"
            
            for child in window.GetChildren():
                info += self.get_window_structure(child, depth + 1, max_depth)
            
            return info
        except Exception as e:
            return f"\n{indent}<Error reading control: {e}>"

    def get_all_window_titles_string(self) -> str:
        """
        Helper to get a simple newline-separated string of all window titles.
        """
        titles = []
        try:
            root = self.get_root_control()
            for window in root.GetChildren():
                try:
                    name = window.Name
                    if name:
                        titles.append(name)
                except: pass
        except Exception:
            titles.append("Error listing windows")
        return "\n".join([f"- {t}" for t in titles])
