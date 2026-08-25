// bluetooth_scanner.go
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"regexp"
	"time"

	"github.com/go-ble/ble"
	"github.com/go-ble/ble/examples/lib/dev"
)

type DeviceInfo struct {
	Address   string   `json:"address"`
	Name      string   `json:"name"`
	RSSI      int      `json:"rssi"`
	Timestamp string   `json:"timestamp"`
	Services  []string `json:"services"`
}

type Scanner struct {
	timeout      time.Duration
	filterRegex  *regexp.Regexp
	historyFile  string
	devices      []DeviceInfo
	history      []DeviceInfo
}

func NewScanner(timeout int, filter, history string) *Scanner {
	var re *regexp.Regexp
	if filter != "" {
		re = regexp.MustCompile(filter)
	}
	s := &Scanner{
		timeout:     time.Duration(timeout) * time.Second,
		filterRegex: re,
		historyFile: history,
	}
	s.loadHistory()
	return s
}

func (s *Scanner) loadHistory() {
	if s.historyFile == "" {
		return
	}
	data, err := ioutil.ReadFile(s.historyFile)
	if err != nil {
		if os.IsNotExist(err) {
			return
		}
		log.Fatal(err)
	}
	json.Unmarshal(data, &s.history)
}

func (s *Scanner) saveHistory() {
	if s.historyFile == "" {
		return
	}
	data, _ := json.MarshalIndent(s.history, "", "  ")
	ioutil.WriteFile(s.historyFile, data, 0644)
}

func (s *Scanner) filter(device ble.Device) bool {
	if s.filterRegex == nil {
		return true
	}
	return s.filterRegex.MatchString(device.Name())
}

func (s *Scanner) scan() {
	fmt.Printf("Scanning for %v...\n", s.timeout)
	d, err := dev.NewDevice("default")
	if err != nil {
		log.Fatal(err)
	}
	ble.SetDefaultDevice(d)

	// Scan with handler
	ch := make(chan bool)
	ble.Scan(context.Background(), true, func(adv ble.Advertisement) {
		if s.filter(adv) {
			info := DeviceInfo{
				Address:   adv.Addr().String(),
				Name:      adv.LocalName(),
				RSSI:      adv.RSSI(),
				Timestamp: time.Now().Format(time.RFC3339),
				Services:  adv.Services(),
			}
			s.devices = append(s.devices, info)
		}
	}, nil)

	time.Sleep(s.timeout)
	ble.StopScan()
	fmt.Printf("Found %d devices:\n", len(s.devices))
	for _, d := range s.devices {
		fmt.Printf("  %s (%s) RSSI: %d dBm\n", d.Name, d.Address, d.RSSI)
	}
	if s.historyFile != "" {
		s.history = append(s.history, s.devices...)
		s.saveHistory()
		fmt.Printf("History updated in %s\n", s.historyFile)
	}
}

func main() {
	timeout := flag.Int("timeout", 10, "scan duration in seconds")
	filter := flag.String("filter", "", "regex filter for device name")
	history := flag.String("history", "", "JSON history file")
	export := flag.String("export", "", "CSV export (requires --history)")
	flag.Parse()

	scanner := NewScanner(*timeout, *filter, *history)
	scanner.scan()

	if *export != "" && *history != "" {
		// Write CSV
		csvFile, _ := os.Create(*export)
		defer csvFile.Close()
		csvFile.WriteString("timestamp,name,address,rssi,services\n")
		for _, e := range scanner.history {
			line := fmt.Sprintf("%s,%s,%s,%d,\"%s\"\n", e.Timestamp, e.Name, e.Address, e.RSSI, strings.Join(e.Services, ";"))
			csvFile.WriteString(line)
		}
		fmt.Printf("History exported to %s\n", *export)
	}
}
