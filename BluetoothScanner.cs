// BluetoothScanner.cs
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using InTheHand.Net.Bluetooth;
using InTheHand.Net.Sockets;

namespace BluetoothScanner
{
    class DeviceInfo
    {
        public string Address { get; set; }
        public string Name { get; set; }
        public int RSSI { get; set; }
        public string Timestamp { get; set; }
        public List<string> Services { get; set; }
    }

    class Scanner
    {
        private int timeout;
        private Regex filter;
        private string historyFile;
        private string exportFile;
        private List<DeviceInfo> devices = new List<DeviceInfo>();
        private List<DeviceInfo> history = new List<DeviceInfo>();

        public Scanner(int timeout, string filter, string history, string export)
        {
            this.timeout = timeout;
            if (!string.IsNullOrEmpty(filter))
                this.filter = new Regex(filter, RegexOptions.Compiled);
            this.historyFile = history;
            this.exportFile = export;
            LoadHistory();
        }

        private void LoadHistory()
        {
            if (string.IsNullOrEmpty(historyFile)) return;
            if (File.Exists(historyFile))
            {
                string json = File.ReadAllText(historyFile);
                // Deserialize (Newtonsoft.Json or System.Text.Json) - skip for brevity
            }
        }

        private void SaveHistory()
        {
            if (string.IsNullOrEmpty(historyFile)) return;
            // Serialize and write
        }

        private bool MatchesFilter(string name)
        {
            return filter == null || (name != null && filter.IsMatch(name));
        }

        public async Task ScanAsync()
        {
            Console.WriteLine($"Scanning for {timeout} seconds...");
            var client = new BluetoothClient();
            var devicesFound = new List<BluetoothDeviceInfo>();

            var task = Task.Run(() =>
            {
                client.DiscoverDevicesAsync(255, false, true, true);
                // Actually we need to use the old DiscoverDevices method (blocking) for simplicity
                devicesFound = client.DiscoverDevices(255, false, true, true).ToList();
            });

            if (await Task.WhenAny(task, Task.Delay(timeout * 1000)) == task)
            {
                // completed early
            }
            else
            {
                // timeout
                client.CancelDiscover();
            }

            foreach (var d in devicesFound)
            {
                if (MatchesFilter(d.DeviceName))
                {
                    var info = new DeviceInfo
                    {
                        Address = d.DeviceAddress.ToString(),
                        Name = d.DeviceName ?? "Unknown",
                        RSSI = d.Rssi, // may not be available
                        Timestamp = DateTime.UtcNow.ToString("o"),
                        Services = new List<string>()
                    };
                    devices.Add(info);
                }
            }

            Console.WriteLine($"Found {devices.Count} devices:");
            foreach (var d in devices)
                Console.WriteLine($"  {d.Name} ({d.Address}) RSSI: {d.RSSI} dBm");

            if (!string.IsNullOrEmpty(historyFile))
            {
                history.AddRange(devices);
                SaveHistory();
                Console.WriteLine("History updated.");
            }
        }

        public static async Task Main(string[] args)
        {
            int timeout = 10;
            string filter = null, history = null, export = null;
            for (int i = 0; i < args.Length; i++)
            {
                switch (args[i])
                {
                    case "--timeout": timeout = int.Parse(args[++i]); break;
                    case "--filter": filter = args[++i]; break;
                    case "--history": history = args[++i]; break;
                    case "--export": export = args[++i]; break;
                }
            }
            var scanner = new Scanner(timeout, filter, history, export);
            await scanner.ScanAsync();
            // Export CSV if needed
        }
    }
}
