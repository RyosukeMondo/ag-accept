import json
import os
import platformdirs
from typing import Any, List, Optional

APP_NAME = "ag-accept"
APP_AUTHOR = "RyosukeMondo"

class ConfigService:
    """
    Service for managing application configuration.
    Wraps config file operations and provides typed accessors.
    """
    def __init__(self):
        self.config_dir = platformdirs.user_config_dir(APP_NAME, APP_AUTHOR)
        self.config_path = os.path.join(self.config_dir, "config.json")
        self.default_config = {
            "interval": 1.0,
            "target_window_title": "Antigravity",
            "search_texts_ide": ["Run command?", "Reject", "Accept"],
            "search_texts_agent_manager": ["Accept"],
            "context_text_agent_manager": ["Run command?"],
            "mode": "AgentManager",
            "debug_enabled": False,
            "window_width": 600,
            "window_height": 700
        }
        self.config = self._load_config()

    def _load_config(self) -> dict:
        if not os.path.exists(self.config_path):
            try:
                os.makedirs(self.config_dir, exist_ok=True)
                with open(self.config_path, "w", encoding="utf-8") as f:
                    json.dump(self.default_config, f, indent=4)
            except Exception as e:
                print(f"ConfigService: Error creating default config: {e}")
                return self.default_config.copy()

        try:
            with open(self.config_path, "r", encoding="utf-8") as f:
                user_config = json.load(f)
                # Merge with defaults
                config = self.default_config.copy()
                config.update(user_config)
                return config
        except Exception as e:
            print(f"ConfigService: Error loading config: {e}")
            return self.default_config.copy()

    def save(self) -> None:
        """Saves values to disk."""
        try:
            with open(self.config_path, "w", encoding="utf-8") as f:
                json.dump(self.config, f, indent=4)
        except Exception as e:
            print(f"ConfigService: Error saving config: {e}")

    def reload(self) -> None:
        self.config = self._load_config()

    def get(self, key: str, default: Any = None) -> Any:
        return self.config.get(key, default)

    def set(self, key: str, value: Any) -> None:
        self.config[key] = value

    def get_config_path(self) -> str:
        return self.config_path
    
    # Typed Accessors 
    
    @property
    def interval(self) -> float:
        return float(self.get("interval", 1.0))
    
    @interval.setter
    def interval(self, value: float):
        self.set("interval", value)

    @property
    def mode(self) -> str:
        return self.get("mode", "AgentManager")

    @mode.setter
    def mode(self, value: str):
        self.set("mode", value)

    @property
    def debug_enabled(self) -> bool:
        return bool(self.get("debug_enabled", False))

    @debug_enabled.setter
    def debug_enabled(self, value: bool):
        self.set("debug_enabled", value)

    @property
    def target_window_title(self) -> str:
        return self.get("target_window_title", "Antigravity")
        
    @property
    def search_texts_ide(self) -> List[str]:
        return self.get("search_texts_ide", [])

    @property
    def search_texts_agent_manager(self) -> List[str]:
        return self.get("search_texts_agent_manager", [])

    @property
    def context_text_agent_manager(self) -> List[str]:
        return self.get("context_text_agent_manager", [])
