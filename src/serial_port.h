#pragma once

#include <optional>
#include <string>

class SerialPort {
public:
    static constexpr int BAUD_RATE = 115200;
    static constexpr int READ_TIMEOUT_MS = 100;

    SerialPort() = default;
    ~SerialPort();

    SerialPort(const SerialPort&) = delete;
    SerialPort& operator=(const SerialPort&) = delete;
    SerialPort(SerialPort&& other) noexcept;
    SerialPort& operator=(SerialPort&& other) noexcept;

    bool open(const std::string& device_path);
    void close();
    bool is_open() const;

    bool write_line(const std::string& data);
    std::optional<std::string> read_line();

    int fd() const { return fd_; }

private:
    int fd_ = -1;
};
