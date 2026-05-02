#include "bridge/alsa_media_port.h"
#include "logger.h"
#include <cstring>

AlsaMediaPort::AlsaMediaPort(RingBuffer<int16_t>& capture_buf,
                             RingBuffer<int16_t>& playback_buf)
    : capture_buf_(capture_buf), playback_buf_(playback_buf) {}

void AlsaMediaPort::create() {
    pj::MediaFormatAudio fmt;
    fmt.init(PJMEDIA_FORMAT_PCM,
             BRIDGE_SAMPLE_RATE,
             BRIDGE_CHANNELS,
             BRIDGE_FRAME_TIME_US,
             BRIDGE_BITS_PER_SAMPLE);
    createPort("alsa_bridge", fmt);
}

void AlsaMediaPort::onFrameRequested(pj::MediaFrame& frame) {
    size_t byte_count = frame.size;
    size_t samples_needed = byte_count / sizeof(int16_t);

    frame.buf.resize(byte_count);
    auto* buf = reinterpret_cast<int16_t*>(frame.buf.data());

    size_t read_count = capture_buf_.read(buf, samples_needed);
    if (read_count < samples_needed) {
        std::memset(buf + read_count, 0,
                    (samples_needed - read_count) * sizeof(int16_t));
    }
    frame.type = PJMEDIA_FRAME_TYPE_AUDIO;
}

void AlsaMediaPort::onFrameReceived(pj::MediaFrame& frame) {
    if (frame.type != PJMEDIA_FRAME_TYPE_AUDIO) return;

    size_t sample_count = frame.size / sizeof(int16_t);
    auto* buf = reinterpret_cast<const int16_t*>(frame.buf.data());

    playback_buf_.try_write(buf, sample_count);
}
