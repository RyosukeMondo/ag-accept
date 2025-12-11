
import pytest
import os
import json
from ag_accept.services.config_service import ConfigService

def test_config_defaults(tmp_path):
    # Setup
    service = ConfigService()
    service.config_path = os.path.join(str(tmp_path), "config.json")
    
    # Test
    # Should use defaults if file doesn't exist
    service.reload()
    assert service.get("interval") == 1.0
    assert service.mode == "AgentManager"

def test_config_save_load(tmp_path):
    service = ConfigService()
    service.config_path = os.path.join(str(tmp_path), "config.json")
    
    # Save
    service.set("interval", 5.5)
    service.mode = "IDE"
    service.save()
    
    # Check file
    assert os.path.exists(service.config_path)
    with open(service.config_path, "r") as f:
        data = json.load(f)
        assert data["interval"] == 5.5
        assert data["mode"] == "IDE"
        
    # Reload new instance
    service2 = ConfigService()
    service2.config_path = os.path.join(str(tmp_path), "config.json")
    service2.reload()
    assert service2.interval == 5.5
    assert service2.mode == "IDE"

def test_typed_accessors(tmp_path):
    service = ConfigService()
    service.config_path = os.path.join(str(tmp_path), "config.json")
    service.reload()
    
    service.debug_enabled = True
    assert service.get("debug_enabled") is True
    
    service.interval = 10.0
    assert service.get("interval") == 10.0
