
import pytest
from unittest.mock import ANY
from ag_accept.services.automation_service import AutomationService
# We don't import strategies here directly to avoid threading issues in tests, 
# relying on mocks via DI or logic checks.

def test_automation_lifecycle(injector):
    service = injector.get(AutomationService)
    
    assert not service.is_running()
    
    # Mock self logger
    logger = lambda x: None
    
    # Start
    # This might spawn a thread, so we should be careful.
    # AutomationService uses 'threading.Thread'. We can mock 'threading.Thread' in the service?
    # Or just start and stop quickly.
    
    service.start_automation("AgentManager", logger)
    assert service.is_running()
    
    service.trigger_snapshot()
    assert service.snapshot_event.is_set()
    
    service.stop_automation()
    assert not service.is_running()
    assert service.stop_event.is_set()

def test_unknown_mode(injector):
    service = injector.get(AutomationService)
    logger_mock = lambda msg: print(msg) # simplistic
    
    # This might print error and stop immediately
    service.start_automation("INVALID_MODE", logger_mock)
    
    # Should stop on error
    assert not service.is_running()
