#pragma once

#include <atomic>
#include <cstdint>
#include <string>

struct AudioConfig {
    unsigned int sample_rate   = 8000;
    unsigned int channels      = 1;
    unsigned int period_frames = 160;
    unsigned int buffer_frames = 640;
};

class AudioLoop {
public:
    AudioLoop() = default;
    ~AudioLoop();

    AudioLoop(const AudioLoop&) = delete;
    AudioLoop& operator=(const AudioLoop&) = delete;

    bool open(const std::string& device_name);
    AudioConfig config() const { return config_; }
    void run(std::atomic<bool>& running);
    void close();

private:
    struct impl;
    impl* impl_ = nullptr;
    AudioConfig config_{};
};
