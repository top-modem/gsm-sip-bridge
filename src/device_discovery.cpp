#include "device_discovery.h"
#include "logger.h"

#include <algorithm>
#include <cstring>
#include <dirent.h>
#include <fstream>
#include <string>
#include <vector>

static std::string read_sysfs_attr(const std::string& path) {
    std::ifstream ifs(path);
    std::string value;
    if (ifs.is_open()) {
        std::getline(ifs, value);
        while (!value.empty() && (value.back() == '\n' || value.back() == '\r'))
            value.pop_back();
    }
    return value;
}

static std::vector<std::string> list_dir(const std::string& path) {
    std::vector<std::string> entries;
    DIR* dir = opendir(path.c_str());
    if (!dir) return entries;
    struct dirent* ent;
    while ((ent = readdir(dir)) != nullptr) {
        if (ent->d_name[0] != '.') entries.emplace_back(ent->d_name);
    }
    closedir(dir);
    return entries;
}

static std::string find_tty_under(const std::string& usb_device_path) {
    std::vector<std::string> ports;

    for (const auto& iface : list_dir(usb_device_path)) {
        std::string iface_path = usb_device_path + "/" + iface;
        for (const auto& child : list_dir(iface_path)) {
            if (child.find("ttyUSB") == 0 || child.find("ttyACM") == 0) {
                ports.push_back("/dev/" + child);
            }
            std::string sub_path = iface_path + "/" + child;
            for (const auto& grandchild : list_dir(sub_path)) {
                if (grandchild.find("ttyUSB") == 0 || grandchild.find("ttyACM") == 0) {
                    ports.push_back("/dev/" + grandchild);
                }
            }
        }
    }

    std::sort(ports.begin(), ports.end());
    ports.erase(std::unique(ports.begin(), ports.end()), ports.end());

    // EC20 AT command port is typically the third serial port (index 2)
    if (ports.size() >= 3) return ports[2];
    if (!ports.empty()) return ports.back();
    return {};
}

static std::string find_sound_card_under(const std::string& usb_device_path) {
    for (const auto& iface : list_dir(usb_device_path)) {
        std::string iface_path = usb_device_path + "/" + iface;
        for (const auto& child : list_dir(iface_path)) {
            if (child.find("sound") == 0) {
                std::string sound_path = iface_path + "/" + child;
                for (const auto& card : list_dir(sound_path)) {
                    if (card.find("card") == 0) {
                        std::string card_num = card.substr(4);
                        return "hw:" + card_num + ",0";
                    }
                }
            }
            std::string sub_path = iface_path + "/" + child;
            for (const auto& grandchild : list_dir(sub_path)) {
                if (grandchild.find("sound") == 0) {
                    std::string sound_path = sub_path + "/" + grandchild;
                    for (const auto& card : list_dir(sound_path)) {
                        if (card.find("card") == 0) {
                            std::string card_num = card.substr(4);
                            return "hw:" + card_num + ",0";
                        }
                    }
                }
            }
        }
    }
    return {};
}

std::optional<DeviceInfo> discover_ec20() {
    const std::string usb_devices_path = "/sys/bus/usb/devices";
    auto devices = list_dir(usb_devices_path);

    for (const auto& dev : devices) {
        std::string dev_path = usb_devices_path + "/" + dev;
        std::string vendor = read_sysfs_attr(dev_path + "/idVendor");
        std::string product = read_sysfs_attr(dev_path + "/idProduct");

        if (vendor != "2c7c" || product != "0125") continue;

        std::string serial = find_tty_under(dev_path);
        std::string alsa = find_sound_card_under(dev_path);

        if (!serial.empty() && !alsa.empty()) {
            LOG_INFO("detected EC20 at %s, audio %s", serial.c_str(), alsa.c_str());
            return DeviceInfo{serial, alsa};
        }

        if (!serial.empty()) {
            LOG_WARN("EC20 found at %s but no audio device (UAC may not be enabled)",
                     serial.c_str());
        }
    }

    return std::nullopt;
}
