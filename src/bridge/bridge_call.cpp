#include "bridge/bridge_call.h"
#include "bridge/bridge_account.h"
#include "logger.h"

BridgeCall::BridgeCall(BridgeAccount& owner, int call_id)
    : pj::Call(owner, call_id),
      owner_(owner),
      start_time_(std::chrono::steady_clock::now()) {}

void BridgeCall::onCallState(pj::OnCallStateParam& /*prm*/) {
    pj::CallInfo ci = getInfo();

    switch (ci.state) {
        case PJSIP_INV_STATE_EARLY:
            LOG_INFO("SIP call ringing");
            sip_state_.store(SipCallState::RINGING, std::memory_order_release);
            break;

        case PJSIP_INV_STATE_CONFIRMED: {
            auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                std::chrono::steady_clock::now() - start_time_).count();
            LOG_INFO("SIP call answered (setup: %ldms)", elapsed);
            sip_state_.store(SipCallState::CONFIRMED, std::memory_order_release);
            break;
        }

        case PJSIP_INV_STATE_DISCONNECTED: {
            auto elapsed = std::chrono::duration_cast<std::chrono::seconds>(
                std::chrono::steady_clock::now() - start_time_).count();

            bool was_confirmed = sip_state_.load(std::memory_order_relaxed) == SipCallState::CONFIRMED;
            if (was_confirmed) {
                LOG_INFO("SIP call ended (duration: %lds, reason: %s)",
                         elapsed, ci.lastReason.c_str());
                sip_state_.store(SipCallState::DISCONNECTED, std::memory_order_release);
            } else {
                LOG_WARN("SIP call failed (reason: %s, code: %d)",
                         ci.lastReason.c_str(), ci.lastStatusCode);
                sip_state_.store(SipCallState::FAILED, std::memory_order_release);
            }
            break;
        }

        default:
            break;
    }
}

void BridgeCall::onCallMediaState(pj::OnCallMediaStateParam& /*prm*/) {
    pj::CallInfo ci = getInfo();

    for (unsigned i = 0; i < ci.media.size(); ++i) {
        if (ci.media[i].type != PJMEDIA_TYPE_AUDIO) continue;
        if (ci.media[i].status != PJSUA_CALL_MEDIA_ACTIVE) continue;

        unsigned int ver = media_version_.fetch_add(1, std::memory_order_acq_rel) + 1;
        LOG_INFO("SIP call media active on index %u (version=%u)", i, ver);
        media_connected_.store(true, std::memory_order_release);
        return;
    }
}
