#pragma once

#include <cstdlib>
#include <pty.h>
#include <string>
#include <unistd.h>

struct PtyPair {
    int master_fd = -1;
    int slave_fd = -1;
    std::string slave_name;

    bool create() {
        char name[256];
        if (openpty(&master_fd, &slave_fd, name, nullptr, nullptr) != 0) {
            return false;
        }
        slave_name = name;
        return true;
    }

    void close() {
        if (master_fd >= 0) { ::close(master_fd); master_fd = -1; }
        if (slave_fd >= 0)  { ::close(slave_fd);  slave_fd = -1; }
    }

    ~PtyPair() { close(); }

    PtyPair() = default;
    PtyPair(const PtyPair&) = delete;
    PtyPair& operator=(const PtyPair&) = delete;
};
