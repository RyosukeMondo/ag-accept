
import tkinter as tk
from tkinter import scrolledtext
import threading
import time
import re
from pywinauto import Desktop
from pywinauto.keyboard import send_keys

class AutoAccepter:
    def __init__(self, root):
        self.root = root
        self.root.title("Auto Accepter")
        self.root.geometry("500x400")

        # Control Frame
        control_frame = tk.Frame(root)
        control_frame.pack(pady=10, padx=10, fill=tk.X)

        # Interval
        tk.Label(control_frame, text="Interval (sec):").pack(side=tk.LEFT)
        self.interval_var = tk.StringVar(value="1.0")
        tk.Entry(control_frame, textvariable=self.interval_var, width=10).pack(side=tk.LEFT, padx=5)

        # Start/Stop Buttons
        self.start_btn = tk.Button(control_frame, text="Start", command=self.start_monitoring, bg="#ccffcc")
        self.start_btn.pack(side=tk.LEFT, padx=5)

        self.stop_btn = tk.Button(control_frame, text="Stop", command=self.stop_monitoring, state=tk.DISABLED, bg="#ffcccc")
        self.stop_btn.pack(side=tk.LEFT, padx=5)

        # Log Area
        self.log_area = scrolledtext.ScrolledText(root, state=tk.DISABLED, height=15)
        self.log_area.pack(pady=10, padx=10, fill=tk.BOTH, expand=True)

        self.is_running = False
        self.monitor_thread = None
        self.current_stop_event = None

    def log(self, message):
        timestamp = time.strftime("%H:%M:%S")
        self.log_area.config(state=tk.NORMAL)
        self.log_area.insert(tk.END, f"[{timestamp}] {message}\n")
        self.log_area.see(tk.END)
        self.log_area.config(state=tk.DISABLED)

    def start_monitoring(self):
        try:
            interval = float(self.interval_var.get())
            if interval <= 0:
                raise ValueError("Interval must be positive")
        except ValueError:
            self.log("Error: Invalid interval")
            return

        self.is_running = True
        # Create a new event for this run to avoid race conditions with old threads
        self.current_stop_event = threading.Event()
        
        self.start_btn.config(state=tk.DISABLED)
        self.stop_btn.config(state=tk.NORMAL)
        self.log(f"Started monitoring (Interval: {interval}s)")

        self.monitor_thread = threading.Thread(target=self.run_monitor_loop, args=(interval, self.current_stop_event))
        self.monitor_thread.daemon = True
        self.monitor_thread.start()

    def stop_monitoring(self):
        if self.is_running:
            self.is_running = False
            if self.current_stop_event:
                self.current_stop_event.set()
            self.start_btn.config(state=tk.NORMAL)
            self.stop_btn.config(state=tk.DISABLED)
            self.log("Stopping monitoring...")

    def run_monitor_loop(self, interval, stop_event):
        desktop = Desktop(backend="uia")
        
        while not stop_event.is_set():
            try:
                found_targets = []
                try:
                    windows = desktop.windows()
                    for w in windows:
                        try:
                            title = w.window_text()
                            if "Antigravity" in title:
                                found_targets.append(w)
                        except Exception:
                            continue # Window might have closed
                except Exception as e:
                    self.log(f"Fail: Error listing windows: {e}")
                    stop_event.wait(interval)
                    continue

                if not found_targets:
                    pass
                else:
                    for w in found_targets:
                        try:
                            title = w.window_text()
                            # Updated to check for "Auto Accepter" to prevent self-detection if title changes
                            if "Auto Accepter" in title or "Antigravity Monitor" in title:
                                continue # Skip self
                            
                            self.log(f"Checking window: '{title}'")
                            
                            # Search for "Accept" text in descendants
                            has_accept = False
                            try:
                                descendants = w.descendants()
                                for item in descendants:
                                        t = item.window_text()
                                        target_text = "Run command? Reject AcceptAlt+⏎"
                                        if t and target_text in t:
                                            has_accept = True
                                            self.log(f"Found target text in element: '{t}'")
                                            break
                            except Exception as e:
                                self.log(f"Warning: scanning failed for '{title}': {e}")
                            
                            if has_accept:
                                self.log(f"Refocusing and sending key to '{title}'")
                                try:
                                    w.set_focus()
                                except Exception as focus_err:
                                     self.log(f"Focus warning: {focus_err}")
                                
                                w.type_keys('%{ENTER}', with_spaces=True)
                                self.log("Success: Sent Alt+Enter")
                            else:
                                self.log(f"Skipping '{title}': Target text not found.")
                            
                        except Exception as e:
                            self.log(f"Fail: Operation on '{title}' failed. Error: {e}")

            except Exception as e:
                self.log(f"Error in loop: {e}")

            if stop_event.wait(interval):
                break
            
        self.log("Monitoring stopped.")

def main():
    root = tk.Tk()
    app = AutoAccepter(root)
    root.mainloop()

if __name__ == "__main__":
    main()
