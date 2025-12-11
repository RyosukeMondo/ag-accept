import time
import threading
from typing import Callable

class SchedulerService:
    """
    Service for running a periodic task until a stop event is set.
    """

    def start(self, task: Callable, interval: float, stop_event: threading.Event):
        """
        Runs the 'task' function periodically. 
        Waits 'interval' seconds between iterations.
        Checks 'stop_event' to determine when to exit.
        """
        while not stop_event.is_set():
            try:
                task()
            except Exception as e:
                # We catch exceptions here to prevent the thread from dying entirely on one error,
                # but ideally the task handles its own local errors.
                # Since we don't have a logger injected here, we print or ignore.
                print(f"SchedulerService: Error in task: {e}")
            
            # Wait for the interval or stop signal
            if stop_event.wait(interval):
                break
