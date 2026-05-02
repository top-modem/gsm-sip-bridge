#include "serial_port.h"
#include "logger.h"

#include <cerrno>
#include <cstring>
#include <fcntl.h>
#include <termios.h>
#include <unistd.h>

SerialPort::~SerialPort() {
    close();
}

SerialPort::SerialPort(SerialPort&& other) noexcept : fd_(other.fd_) {
    other.fd_ = -1;
}

SerialPort& SerialPort::operator=(SerialPort&& other) noexcept {
    if (this != &other) {
        close();
        fd_ = other.fd_;
        other.fd_ = -1;
    }
    return *this;
}

bool SerialPort::open(const std::string& device_path) {
    fd_ = ::open(device_path.c_str(), O_RDWR | O_NOCTTY);
    if (fd_ < 0) {
        LOG_ERROR("serial open failed: %s: %s", device_path.c_str(), std::strerror(errno));
        return false;
    }

    struct termios tty{};
    if (tcgetattr(fd_, &tty) != 0) {
        LOG_ERROR("tcgetattr failed: %s", std::strerror(errno));
        close();
        return false;
    }

    cfsetispeed(&tty, B115200);
    cfsetospeed(&tty, B115200);

    tty.c_cflag = (tty.c_cflag & ~CSIZE) | CS8;
    tty.c_cflag &= ~(PARENB | CSTOPB | CRTSCTS);
    tty.c_cflag |= CLOCAL | CREAD;

    tty.c_iflag &= ~(IXON | IXOFF | IXANY | IGNBRK | INLCR | IGNCR | ICRNL);
    tty.c_lflag &= ~(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
    tty.c_oflag &= ~OPOST;

    tty.c_cc[VMIN]  = 0;
    tty.c_cc[VTIME] = 1; // 100ms read timeout

    if (tcsetattr(fd_, TCSANOW, &tty) != 0) {
        LOG_ERROR("tcsetattr failed: %s", std::strerror(errno));
        close();
        return false;
    }

    tcflush(fd_, TCIOFLUSH);
    return true;
}

void SerialPort::close() {
    if (fd_ >= 0) {
        ::close(fd_);
        fd_ = -1;
    }
}

bool SerialPort::is_open() const {
    return fd_ >= 0;
}

bool SerialPort::write_line(const std::string& data) {
    if (fd_ < 0) return false;

    std::string cmd = data + "\r\n";
    ssize_t written = ::write(fd_, cmd.c_str(), cmd.size());
    if (written < 0) {
        LOG_ERROR("serial write failed: %s", std::strerror(errno));
        return false;
    }
    return static_cast<size_t>(written) == cmd.size();
}

std::optional<std::string> SerialPort::read_line() {
    if (fd_ < 0) return std::nullopt;

    std::string line;
    char ch;

    while (true) {
        ssize_t n = ::read(fd_, &ch, 1);
        if (n < 0) {
            if (errno == EAGAIN || errno == EWOULDBLOCK) return std::nullopt;
            LOG_ERROR("serial read failed: %s", std::strerror(errno));
            return std::nullopt;
        }
        if (n == 0) {
            return line.empty() ? std::nullopt : std::optional<std::string>(line);
        }
        if (ch == '\n') {
            if (!line.empty() && line.back() == '\r') {
                line.pop_back();
            }
            if (line.empty()) continue;
            return line;
        }
        line += ch;
    }
}
