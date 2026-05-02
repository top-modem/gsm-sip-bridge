#pragma once

#include "serial_port.h"

#include <functional>
#include <optional>
#include <string>

enum class CallState { IDLE, RINGING, ANSWERED, ECHOING, ENDED };

const char* call_state_str(CallState state);

using UrcHandler = std::function<void(const std::string& line)>;

class AtCommander {
public:
    explicit AtCommander(SerialPort& port);

    bool send(const std::string& command);
    bool send_and_expect_ok(const std::string& command, int timeout_ms = 3000);
    bool answer_call();
    bool hangup();
    bool query_network_registration();

    std::optional<std::string> poll_urc();

    void set_verbose(bool verbose) { verbose_ = verbose; }

private:
    SerialPort& port_;
    bool verbose_ = false;

    std::optional<std::string> read_response(int timeout_ms);
};
