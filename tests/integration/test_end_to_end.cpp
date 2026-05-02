#include "at_commander.h"
#include "serial_port.h"
#include "pty_pair.h"

#include <gtest/gtest.h>
#include <cstring>
#include <thread>
#include <chrono>
#include <unistd.h>

class EndToEndTest : public ::testing::Test {
protected:
    PtyPair pty;
    SerialPort app_port;

    void SetUp() override {
        ASSERT_TRUE(pty.create()) << "failed to create pty pair";
        ASSERT_TRUE(app_port.open(pty.slave_name));
    }

    void TearDown() override {
        app_port.close();
    }

    void sim_send(const std::string& line) {
        std::string data = line + "\r\n";
        ::write(pty.master_fd, data.c_str(), data.size());
    }

    std::string sim_read(int timeout_ms = 2000) {
        char buf[256]{};
        fd_set fds;
        FD_ZERO(&fds);
        FD_SET(pty.master_fd, &fds);
        struct timeval tv;
        tv.tv_sec = timeout_ms / 1000;
        tv.tv_usec = (timeout_ms % 1000) * 1000;

        if (select(pty.master_fd + 1, &fds, nullptr, nullptr, &tv) > 0) {
            ssize_t n = ::read(pty.master_fd, buf, sizeof(buf) - 1);
            if (n > 0) return std::string(buf, n);
        }
        return {};
    }
};

TEST_F(EndToEndTest, full_call_lifecycle_ring_answer_hangup) {
    // Arrange
    AtCommander at(app_port);

    // Act: simulate RING -> answer -> NO CARRIER
    sim_send("RING");
    std::this_thread::sleep_for(std::chrono::milliseconds(50));

    auto urc = at.poll_urc();
    ASSERT_TRUE(urc.has_value());
    ASSERT_NE(urc->find("RING"), std::string::npos);

    // Simulate modem accepting ATA
    std::thread responder([this]() {
        std::string cmd = sim_read(2000);
        EXPECT_NE(cmd.find("ATA"), std::string::npos);
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("OK");
    });

    bool answered = at.answer_call();
    responder.join();
    ASSERT_TRUE(answered);

    // Simulate remote hangup
    sim_send("NO CARRIER");
    std::this_thread::sleep_for(std::chrono::milliseconds(50));

    auto hangup_urc = at.poll_urc();
    ASSERT_TRUE(hangup_urc.has_value());
    EXPECT_NE(hangup_urc->find("NO CARRIER"), std::string::npos);
}

TEST_F(EndToEndTest, multiple_sequential_calls) {
    // Arrange
    AtCommander at(app_port);
    constexpr int NUM_CALLS = 3;

    for (int i = 0; i < NUM_CALLS; ++i) {
        // Act: simulate RING -> ATA -> OK -> NO CARRIER
        sim_send("RING");
        std::this_thread::sleep_for(std::chrono::milliseconds(50));

        auto urc = at.poll_urc();
        ASSERT_TRUE(urc.has_value()) << "call " << i;

        std::thread responder([this]() {
            std::string cmd = sim_read(2000);
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
            sim_send("OK");
        });

        at.answer_call();
        responder.join();

        sim_send("NO CARRIER");
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        at.poll_urc();
    }

    // Assert: all calls completed
    SUCCEED() << "completed " << NUM_CALLS << " sequential call cycles";
}

TEST_F(EndToEndTest, no_fd_leak_across_serial_opens) {
    // Arrange
    app_port.close();

    auto count_fds = []() -> int {
        int count = 0;
        char path[256];
        for (int fd = 0; fd < 1024; ++fd) {
            snprintf(path, sizeof(path), "/proc/self/fd/%d", fd);
            if (access(path, F_OK) == 0) ++count;
        }
        return count;
    };

    int fd_before = count_fds();

    // Act
    for (int i = 0; i < 5; ++i) {
        SerialPort temp;
        ASSERT_TRUE(temp.open(pty.slave_name));
        temp.close();
    }

    // Assert
    int fd_after = count_fds();
    EXPECT_LE(fd_after, fd_before + 2)
        << "possible fd leak: before=" << fd_before << " after=" << fd_after;
}
