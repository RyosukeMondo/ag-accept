import os

class DebugService:
    """
    Service for saving and viewing debug snapshots.
    """
    
    def __init__(self, debug_filename: str = "debug_snapshot.txt"):
        self.debug_filename = debug_filename

    def save_snapshot(self, content: str) -> None:
        """
        Saves the given string content to a file.
        """
        try:
            with open(self.debug_filename, "w", encoding="utf-8") as f:
                f.write(content)
        except Exception as e:
            print(f"DebugService: Failed to save snapshot: {e}")

    def open_snapshot(self) -> None:
        """
        Opens the snapshot file using the OS default application.
        """
        try:
            os.startfile(self.debug_filename)
        except Exception as e:
            print(f"DebugService: Failed to open snapshot: {e}")
