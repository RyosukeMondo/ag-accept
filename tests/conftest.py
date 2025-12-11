
import pytest
from injector import Injector
from unittest.mock import MagicMock
import sys
import os

# Add src to path so we can import modules
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../src')))

from ag_accept.di_module import AppModule
from ag_accept.services.window_service import WindowService
from ag_accept.services.text_query_service import TextQueryService
from ag_accept.services.scheduler_service import SchedulerService
from ag_accept.services.debug_service import DebugService
from ag_accept.services.config_service import ConfigService

@pytest.fixture
def mock_window_service():
    return MagicMock(spec=WindowService)

@pytest.fixture
def mock_text_service():
    return MagicMock(spec=TextQueryService)

@pytest.fixture
def mock_scheduler_service():
    return MagicMock(spec=SchedulerService)

@pytest.fixture
def mock_debug_service():
    return MagicMock(spec=DebugService)

@pytest.fixture
def mock_config_service(tmp_path):
    # Create a real but temporary config service, or mock it?
    # Real is better for logic testing, but we can mock the file path
    service = ConfigService()
    # Override path to temp dir
    service.config_dir = str(tmp_path)
    service.config_path = os.path.join(str(tmp_path), "test_config.json")
    # Reset config to defaults
    service.config = service.default_config.copy()
    return service

@pytest.fixture
def injector(mock_window_service, mock_text_service, mock_scheduler_service, mock_debug_service, mock_config_service):
    # Custom injector that binds mocks
    def configure_mocks(binder):
        binder.bind(WindowService, to=mock_window_service)
        binder.bind(TextQueryService, to=mock_text_service)
        binder.bind(SchedulerService, to=mock_scheduler_service)
        binder.bind(DebugService, to=mock_debug_service)
        binder.bind(ConfigService, to=mock_config_service)
        
    return Injector([configure_mocks])
