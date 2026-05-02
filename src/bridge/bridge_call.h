#pragma once

#include <pjsua2.hpp>
#include <atomic>
#include <chrono>

class BridgeAccount;

enum class SipCallState {
    DIALING,
    RINGING,
    CONFIRMED,
    DISCONNECTED,
    FAILED
};

class BridgeCall : public pj::Call {
public:
    BridgeCall(BridgeAccount& owner, int call_id = PJSUA_INVALID_ID);

    void onCallState(pj::OnCallStateParam& prm) override;
    void onCallMediaState(pj::OnCallMediaStateParam& prm) override;

    SipCallState sip_state() const {
        return sip_state_.load(std::memory_order_acquire);
    }

    bool media_connected() const {
        return media_connected_.load(std::memory_order_acquire);
    }

private:
    BridgeAccount& owner_;
    std::atomic<SipCallState> sip_state_{SipCallState::DIALING};
    std::atomic<bool> media_connected_{false};
    std::chrono::steady_clock::time_point start_time_;
};
