
import sys
import os
import tkinter as tk

# Add src to path
sys.path.append(os.path.join(os.getcwd(), "src"))

try:
    print("Verifying DI Setup...")
    from injector import Injector
    from ag_accept.di_module import AppModule
    from ag_accept.services.config_service import ConfigService
    from ag_accept.services.automation_service import AutomationService
    from ag_accept.ui import AutoAccepterUI
    
    injector = Injector([AppModule])
    
    print("Resolving ConfigService...")
    config = injector.get(ConfigService)
    print(f"Config Loaded: {config.get_config_path()}")
    
    print("Resolving AutomationService...")
    automation = injector.get(AutomationService)
    print(f"Automation service initialized: {automation}")
    
    print("Resolving AutoAccepterUI...")
    # Bind root
    root = tk.Tk()
    injector.binder.bind(tk.Tk, to=root, scope=None)
    
    ui = injector.get(AutoAccepterUI)
    print("UI resolved successfully.")
    
    print("DI VERIFICATION SUCCESSFUL!")
    root.destroy()
except Exception as e:
    print(f"VERIFICATION FAILED: {e}")
    import traceback
    traceback.print_exc()
