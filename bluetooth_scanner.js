// bluetooth_scanner.js
const noble = require('@abandonware/noble');
const fs = require('fs');
const { program } = require('commander');
const { performance } = require('perf_hooks');

class BluetoothScanner {
    constructor(options) {
        this.timeout = options.timeout || 10;
        this.filterPattern = options.filter ? new RegExp(options.filter) : null;
        this.historyFile = options.history || null;
        this.devices = [];
        this.history = this.loadHistory();
        this.startTime = null;
    }

    loadHistory() {
        if (this.historyFile && fs.existsSync(this.historyFile)) {
            try {
                return JSON.parse(fs.readFileSync(this.historyFile));
            } catch (e) {
                return [];
            }
        }
        return [];
    }

    saveHistory() {
        if (this.historyFile) {
            fs.writeFileSync(this.historyFile, JSON.stringify(this.history, null, 2));
        }
    }

    deviceFilter(device) {
        if (this.filterPattern && device.localName) {
            return this.filterPattern.test(device.localName);
        }
        return true;
    }

    startScan() {
        return new Promise((resolve) => {
            console.log(`Scanning for ${this.timeout} seconds...`);
            this.startTime = performance.now();
            noble.on('discover', (device) => {
                if (this.deviceFilter(device)) {
                    const info = {
                        address: device.address,
                        name: device.localName || 'Unknown',
                        rssi: device.rssi,
                        timestamp: new Date().toISOString(),
                        services: device.advertisement.serviceUuids || []
                    };
                    this.devices.push(info);
                }
            });

            noble.startScanning([], true);
            setTimeout(() => {
                noble.stopScanning();
                console.log(`Found ${this.devices.length} devices:`);
                this.devices.forEach(d => {
                    console.log(`  ${d.name} (${d.address}) RSSI: ${d.rssi} dBm`);
                });
                // Save history
                if (this.historyFile) {
                    this.history = this.history.concat(this.devices);
                    this.saveHistory();
                    console.log(`History updated in ${this.historyFile}`);
                }
                resolve(this.devices);
            }, this.timeout * 1000);
        });
    }
}

// Command line arguments
program
    .option('-t, --timeout <seconds>', 'Scan duration', parseInt, 10)
    .option('-f, --filter <regex>', 'Filter by device name')
    .option('-h, --history <file>', 'History JSON file')
    .option('-e, --export <file>', 'Export history to CSV (requires --history)')
    .parse(process.argv);

const opts = program.opts();
const scanner = new BluetoothScanner(opts);
scanner.startScan().then(() => {
    if (opts.export && opts.history) {
        const csvLines = ['timestamp,name,address,rssi,services'];
        scanner.history.forEach(entry => {
            csvLines.push(`${entry.timestamp},${entry.name},${entry.address},${entry.rssi},"${entry.services.join(';')}"`);
        });
        fs.writeFileSync(opts.export, csvLines.join('\n'));
        console.log(`History exported to ${opts.export}`);
    }
});
