# Feature Specification: GSM to SIP Audio Bridge

**Feature Branch**: `003-gsm-sip-bridge`
**Created**: 2026-05-02
**Status**: Draft
**Input**: User description: "the program should listen on both sip and the gsm lines, when a gsm incoming call is landed, attend it, and then dial a sip number (configurable via config.ini), and route the audio in both the direction. during the time sip is dialed, play a beep pattern in the gsm side, as feedback for the called. make 599 as the default entry in the sip side (to forward)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - GSM Incoming Call Bridges to SIP (Priority: P1)

A caller dials the GSM number attached to the EC20 module. The system auto-answers the GSM call and immediately initiates an outbound SIP call to a preconfigured SIP extension (default: 599). While the SIP call is ringing, the GSM caller hears a repeating beep pattern as audible feedback that the call is being connected. Once the SIP party answers, the beep stops and audio flows bidirectionally: the GSM caller hears the SIP party and the SIP party hears the GSM caller. When either party hangs up, both legs of the call are terminated.

**Why this priority**: This is the complete end-to-end value proposition. Without the bridge working, the feature delivers nothing.

**Independent Test**: Call the GSM number, hear beeps while the SIP side rings, then have a two-way conversation with the SIP party.

**Acceptance Scenarios**:

1. **Given** the system is running with valid GSM and SIP connections, **When** an incoming GSM call arrives, **Then** the system answers it automatically and initiates an outbound SIP call to the configured extension.
2. **Given** the GSM call is answered and the SIP call is ringing, **When** the GSM caller listens, **Then** they hear a repeating beep pattern indicating the call is being connected.
3. **Given** the SIP party answers the call, **When** either party speaks, **Then** the other party hears them clearly with minimal delay.
4. **Given** a bridged call is active, **When** the GSM caller hangs up, **Then** the SIP leg is also terminated.
5. **Given** a bridged call is active, **When** the SIP party hangs up, **Then** the GSM leg is also terminated.

---

### User Story 2 - Bridge Configuration (Priority: P2)

An operator configures the bridge behavior through the existing config.ini file. The configuration specifies which SIP extension to dial when a GSM call arrives. The system reads this on startup and uses it for all bridged calls.

**Why this priority**: Without configuration, the SIP destination is hardcoded. Configuration enables deployment flexibility but the bridge itself must work first.

**Independent Test**: Change the SIP destination in config.ini, restart, call the GSM number, and verify the call reaches the new destination.

**Acceptance Scenarios**:

1. **Given** the config.ini contains a `[bridge]` section with `sip_destination = 599`, **When** a GSM call arrives, **Then** the system dials SIP extension 599.
2. **Given** the config.ini contains `sip_destination = 100`, **When** a GSM call arrives, **Then** the system dials SIP extension 100 instead of the default.
3. **Given** the config.ini has no `[bridge]` section, **When** the system starts, **Then** it uses extension 599 as the default SIP destination.

---

### User Story 3 - Continuous Operation and Error Recovery (Priority: P3)

The bridge operates continuously, handling multiple sequential GSM-to-SIP calls. If the SIP call fails to connect (busy, no answer, network error), the system plays an error tone to the GSM caller and hangs up both legs. The system then returns to idle and is ready for the next GSM call.

**Why this priority**: Resilience is needed for production use, but core bridging must work first.

**Independent Test**: Make multiple calls, simulate SIP unreachable, and verify the system recovers for each subsequent call.

**Acceptance Scenarios**:

1. **Given** a bridged call has ended, **When** a new GSM call arrives, **Then** the system bridges it to SIP without requiring a restart.
2. **Given** the system dials the SIP extension but it is busy or unreachable, **When** the SIP call fails, **Then** the GSM caller hears a distinct error tone and the GSM call is terminated.
3. **Given** the system receives SIGINT/SIGTERM during an active bridged call, **When** the signal is received, **Then** both call legs are terminated and the system shuts down cleanly.
4. **Given** the SIP registration is lost during operation, **When** the system detects the loss, **Then** it re-registers automatically and continues accepting GSM calls once registered.

---

### Edge Cases

- What happens when a GSM call arrives while a bridge is already active? The system ignores the second GSM call (the EC20 handles call waiting rejection at the modem level).
- What happens when the SIP party does not answer within a timeout? The system terminates the SIP attempt after 30 seconds, plays an error tone to the GSM caller, and hangs up the GSM call.
- What happens when audio quality degrades on one leg? The system continues bridging without intervention; audio quality issues are passed through transparently.
- What happens when the GSM module disconnects from the network mid-call? The system detects the loss via the AT command interface, terminates the SIP leg, and returns to idle.
- What happens when the beep pattern is playing and the SIP party answers? The beep stops immediately and bidirectional audio begins.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST listen simultaneously on both the GSM module (via AT commands and ALSA audio) and the SIP server (via SIP registration).
- **FR-002**: System MUST auto-answer incoming GSM calls and initiate an outbound SIP call to the configured destination extension.
- **FR-003**: System MUST play a repeating beep pattern to the GSM caller while the outbound SIP call is ringing.
- **FR-004**: System MUST stop the beep and establish bidirectional audio bridging when the SIP party answers.
- **FR-005**: System MUST route audio from the GSM leg to the SIP leg and from the SIP leg to the GSM leg simultaneously.
- **FR-006**: System MUST terminate both call legs when either party hangs up.
- **FR-007**: System MUST read the SIP destination extension from the `[bridge]` section of config.ini, defaulting to 599 if absent.
- **FR-008**: System MUST handle SIP call failure (busy, timeout, error) by playing an error tone to the GSM caller and terminating the GSM call.
- **FR-009**: System MUST handle sequential calls without requiring a restart.
- **FR-010**: System MUST log all significant events (GSM ring, SIP dial, SIP answer, bridge established, call ended, errors) with timestamps.
- **FR-011**: System MUST shut down gracefully on SIGINT/SIGTERM, terminating any active call legs and de-registering from the SIP server.

### Key Entities

- **BridgeConfig**: The bridge-specific configuration: SIP destination extension (default: 599). Read from the `[bridge]` section of the shared config.ini file.
- **BridgedCall**: Represents an active GSM-to-SIP bridged call: GSM call state, SIP call state, bridge state (dialing, ringing, bridged, ended), and start timestamp. Only one bridged call exists at a time.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: GSM calls are answered within 2 seconds of the first RING.
- **SC-002**: The outbound SIP call is initiated within 1 second of answering the GSM call.
- **SC-003**: The GSM caller hears beeps within 500ms of the GSM call being answered.
- **SC-004**: Bidirectional audio is established within 500ms of the SIP party answering.
- **SC-005**: End-to-end voice latency through the bridge is less than 300ms.
- **SC-006**: Both call legs are terminated within 2 seconds of either party hanging up.
- **SC-007**: System handles at least 50 sequential bridged calls without a restart or resource leak.

## Assumptions

- The EC20 GSM module and SIP server are both available and configured (from features 001 and 002).
- The existing config.ini file is extended with a `[bridge]` section; the `[sip]` section from feature 002 provides SIP credentials.
- The GSM audio interface is the EC20's USB Audio Class device (ALSA), and the SIP audio interface is managed by PJSIP's conference bridge.
- Audio bridging requires converting between ALSA PCM frames (8kHz S16_LE mono from the EC20) and PJSIP's media port format (G.711 codec).
- The beep pattern is a simple tone (e.g., 400Hz, 200ms on / 200ms off) generated programmatically, not a pre-recorded file.
- The program runs as a single binary combining both GSM and SIP functionality.
- Only one GSM-to-SIP bridge is active at a time; the EC20 supports one voice call at a time.
