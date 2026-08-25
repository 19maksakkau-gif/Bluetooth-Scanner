

## 1. Python (`bluetooth_scanner.py`)

```python
# bluetooth_scanner.py
import asyncio
import json
import re
import argparse
from datetime import datetime
from bleak import BleakScanner

class BluetoothScanner:
    def __init__(self, timeout=10, filter_pattern=None, history_file=None):
        self.timeout = timeout
        self.filter_pattern = re.compile(filter_pattern) if filter_pattern else None
        self.history_file = history_file
        self.devices = []
        self.history = self.load_history() if history_file else []

    def load_history(self):
        try:
            with open(self.history_file, 'r') as f:
                return json.load(f)
        except (FileNotFoundError, json.JSONDecodeError):
            return []

    def save_history(self):
        if self.history_file:
            with open(self.history_file, 'w') as f:
                json.dump(self.history, f, indent=2)

    def device_filter(self, device):
        if self.filter_pattern and device.name:
            return self.filter_pattern.search(device.name) is not None
        return True  # no filter or no name

    async def scan(self):
        def callback(device, advertisement_data):
            if self.device_filter(device):
                info = {
                    "address": device.address,
                    "name": device.name or "Unknown",
                    "rssi": advertisement_data.rssi,
                    "timestamp": datetime.now().isoformat(),
                    "services": [str(s) for s in advertisement_data.service_uuids] if advertisement_data.service_uuids else []
                }
                self.devices.append(info)

        scanner = BleakScanner(callback)
        await scanner.start()
        await asyncio.sleep(self.timeout)
        await scanner.stop()
        return self.devices

    def run(self):
        print(f"Scanning for {self.timeout} seconds...")
        loop = asyncio.get_event_loop()
        devices = loop.run_until_complete(self.scan())
        print(f"Found {len(devices)} devices:")
        for d in devices:
            print(f"  {d['name']} ({d['address']}) RSSI: {d['rssi']} dBm")
        # Append to history
        if self.history_file:
            self.history.extend(devices)
            self.save_history()
            print(f"History updated in {self.history_file}")
        return devices

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--timeout", type=int, default=10, help="Scan duration in seconds")
    parser.add_argument("--filter", help="Regex to filter device names")
    parser.add_argument("--history", help="JSON file to store history")
    parser.add_argument("--export", help="CSV file to export history (if --history is used)")
    args = parser.parse_args()

    scanner = BluetoothScanner(args.timeout, args.filter, args.history)
    scanner.run()

    if args.export and args.history:
        import csv
        with open(args.export, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=["timestamp", "name", "address", "rssi", "services"])
            writer.writeheader()
            writer.writerows(scanner.history)
        print(f"History exported to {args.export}")
