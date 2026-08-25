// bluetooth_scanner.cpp
#include <iostream>
#include <vector>
#include <string>
#include <regex>
#include <thread>
#include <chrono>
#include <fstream>
#include <json/json.h> // using jsoncpp
#include <bluetooth/bluetooth.h>
#include <bluetooth/hci.h>
#include <bluetooth/hci_lib.h>
#include <unistd.h>

struct DeviceInfo {
    std::string address;
    std::string name;
    int rssi;
    std::string timestamp;
    std::vector<std::string> services;
};

class Scanner {
private:
    int timeout;
    std::regex filter;
    bool hasFilter;
    std::string historyFile;
    std::string exportFile;
    std::vector<DeviceInfo> devices;
    std::vector<DeviceInfo> history;

public:
    Scanner(int t, const std::string& f, const std::string& h, const std::string& e)
        : timeout(t), hasFilter(!f.empty()), historyFile(h), exportFile(e) {
        if (hasFilter) filter = std::regex(f, std::regex::icase);
        loadHistory();
    }

    void loadHistory() {
        if (historyFile.empty()) return;
        std::ifstream ifs(historyFile);
        if (ifs) {
            Json::Value root;
            ifs >> root;
            // parse history
        }
    }

    void saveHistory() {
        if (historyFile.empty()) return;
        Json::Value root(Json::arrayValue);
        for (auto& d : history) {
            Json::Value item;
            item["address"] = d.address;
            item["name"] = d.name;
            item["rssi"] = d.rssi;
            item["timestamp"] = d.timestamp;
            Json::Value svcs(Json::arrayValue);
            for (auto& s : d.services) svcs.append(s);
            item["services"] = svcs;
            root.append(item);
        }
        std::ofstream ofs(historyFile);
        ofs << root;
    }

    bool matchesFilter(const std::string& name) {
        return !hasFilter || std::regex_search(name, filter);
    }

    void scan() {
        std::cout << "Scanning for " << timeout << " seconds..." << std::endl;
        int dev_id = hci_get_route(nullptr);
        int sock = hci_open_dev(dev_id);
        if (sock < 0) {
            perror("hci_open_dev");
            return;
        }
        int len = 8;
        int max_rsp = 255;
        char buf[HCI_MAX_EVENT_SIZE];
        struct hci_inquiry_req ir = { 0 };
        ir.dev_id = dev_id;
        ir.num_rsp = max_rsp;
        ir.length = timeout;
        ir.flags = IREQ_CACHE_FLUSH;

        auto rsps = (inquiry_info*)malloc(max_rsp * sizeof(inquiry_info));
        int count = hci_inquiry(dev_id, len, max_rsp, nullptr, &rsps, IREQ_CACHE_FLUSH);
        if (count < 0) perror("hci_inquiry");
        else {
            for (int i = 0; i < count; i++) {
                char addr[19];
                ba2str(&rsps[i].bdaddr, addr);
                char name[248] = {0};
                if (hci_read_remote_name(sock, &rsps[i].bdaddr, sizeof(name), name, 0) < 0)
                    strcpy(name, "Unknown");
                if (matchesFilter(name)) {
                    DeviceInfo info;
                    info.address = addr;
                    info.name = name;
                    info.rssi = rsps[i].rssi;
                    info.timestamp = std::to_string(time(nullptr));
                    devices.push_back(info);
                }
            }
        }
        close(sock);
        free(rsps);

        std::cout << "Found " << devices.size() << " devices:" << std::endl;
        for (auto& d : devices) {
            std::cout << "  " << d.name << " (" << d.address << ") RSSI: " << d.rssi << " dBm" << std::endl;
        }
        if (!historyFile.empty()) {
            history.insert(history.end(), devices.begin(), devices.end());
            saveHistory();
            std::cout << "History updated." << std::endl;
        }
    }
};

int main(int argc, char* argv[]) {
    int timeout = 10;
    std::string filter, history, exportFile;
    for (int i = 1; i < argc; i++) {
        std::string arg = argv[i];
        if (arg == "--timeout" && i+1 < argc) timeout = std::stoi(argv[++i]);
        else if (arg == "--filter" && i+1 < argc) filter = argv[++i];
        else if (arg == "--history" && i+1 < argc) history = argv[++i];
        else if (arg == "--export" && i+1 < argc) exportFile = argv[++i];
    }
    Scanner scanner(timeout, filter, history, exportFile);
    scanner.scan();
    return 0;
}
