
import tkinter as tk
import customtkinter as ctk
from injector import Injector
from ag_accept.di_module import AppModule
from ag_accept.ui import AutoAccepterUI

def main():
    try:
        # Initialize DI
        injector = Injector([AppModule])
        
        # Setup CustomTkinter
        ctk.set_appearance_mode("Dark")  # Modes: "System" (standard), "Dark", "Light"
        ctk.set_default_color_theme("blue")  # Themes: "blue" (standard), "green", "dark-blue"
        
        # Root CTk
        root = ctk.CTk()
        
        # Binding specific instance
        injector.binder.bind(ctk.CTk, to=root, scope=None)
        # Also bind tk.Tk just in case as CTk inherits from it? 
        # Actually CTk inherits from Tk or Widget? 
        # CTk inherits from CTkBaseClass -> tkinter.Tk usually.
        injector.binder.bind(tk.Tk, to=root, scope=None) 
        
        # Now we can resolve UI
        app = injector.get(AutoAccepterUI)
        
        root.mainloop()
    except Exception as e:
        import traceback
        with open("crash.log", "a") as f:
            f.write(f"\nCRASH IN MAIN LOOP/INIT:\n{traceback.format_exc()}\n")
        raise e

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        import traceback
        with open("crash.log", "w") as f:
            f.write(traceback.format_exc())
        print("CRASHED! Check crash.log")
        input("Press Enter to exit...")
