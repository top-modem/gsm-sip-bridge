#pragma once

#include "ring_buffer.h"
#include <pjsua2.hpp>
#include <cstdint>

static constexpr unsigned int BRIDGE_SAMPLE_RATE = 8000;
static constexpr unsigned int BRIDGE_CHANNELS = 1;
static constexpr unsigned int BRIDGE_FRAME_TIME_US = 20000;
static constexpr unsigned int BRIDGE_BITS_PER_SAMPLE = 16;
static constexpr unsigned int BRIDGE_PTIME_MS = 20;
static constexpr unsigned int BRIDGE_FRAME_SAMPLES = BRIDGE_SAMPLE_RATE * BRIDGE_PTIME_MS / 1000;

class AlsaMediaPort : public pj::AudioMediaPort {
public:
    AlsaMediaPort(RingBuffer<int16_t>& capture_buf,
                  RingBuffer<int16_t>& playback_buf);

    void create();

    void onFrameRequested(pj::MediaFrame& frame) override;
    void onFrameReceived(pj::MediaFrame& frame) override;

private:
    RingBuffer<int16_t>& capture_buf_;
    RingBuffer<int16_t>& playback_buf_;
};
