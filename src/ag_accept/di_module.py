
from injector import Module, Binder, singleton

from ag_accept.services.config_service import ConfigService
from ag_accept.services.window_service import WindowService
from ag_accept.services.text_query_service import TextQueryService
from ag_accept.services.scheduler_service import SchedulerService
from ag_accept.services.debug_service import DebugService
from ag_accept.services.automation_service import AutomationService

class AppModule(Module):
    def configure(self, binder: Binder) -> None:
        """
        Bind all services as singletons.
        """
        binder.bind(ConfigService, scope=singleton)
        binder.bind(WindowService, scope=singleton)
        binder.bind(TextQueryService, scope=singleton)
        binder.bind(SchedulerService, scope=singleton)
        binder.bind(DebugService, scope=singleton)
        
        # AutomationService depends on specific instances of others, but Injector resolves recursively
        binder.bind(AutomationService, scope=singleton)
