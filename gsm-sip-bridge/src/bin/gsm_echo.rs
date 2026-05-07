use clap::Parser;
use gsm_sip_bridge::modules::at_commander::{AtCommander, AtResponse};
use gsm_sip_bridge::modules::discovery;
use gsm_sip_bridge::observability::logging;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "gsm-echo",
    version,
    about = "Debug tool: single-card GSM audio loopback (no SIP). \
             Answers incoming calls and loops captured audio back to the caller."
)]
struct Cli {
    #[arg(short = 's', long = "serial", requires = "audio")]
    serial: Option<PathBuf>,

    #[arg(short = 'a', long = "audio", requires = "serial")]
    audio: Option<String>,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    logging::init(cli.verbose);

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "starting gsm-echo (GSM audio loopback)"
    );

    let (serial_port, audio_device) = match resolve_module(&cli) {
        Ok(pair) => pair,
        Err(e) => {
            tracing::error!(error = %e, "module resolution failed");
            return ExitCode::from(1);
        }
    };

    tracing::info!(
        serial = %serial_port.display(),
        audio = %audio_device,
        "using module"
    );

    let mut at = match AtCommander::open(&serial_port) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, "failed to open serial port");
            return ExitCode::from(1);
        }
    };

    if let Err(e) = probe_module(&mut at) {
        tracing::error!(error = %e, "module probe failed");
        return ExitCode::from(1);
    }

    configure_module(&mut at);

    if let Ok((rssi, ber)) = at.check_signal() {
        tracing::info!(rssi, ber, "signal quality");
    }

    tracing::info!(
        "monitoring for incoming calls (Ctrl+C to quit). \
         Audio is bridged through PJSIP sound device on: {audio_device}"
    );

    let exit = run_event_loop(&mut at);

    tracing::info!("gsm-echo stopped");
    exit
}

fn resolve_module(cli: &Cli) -> Result<(PathBuf, String), String> {
    if let (Some(serial), Some(audio)) = (&cli.serial, &cli.audio) {
        return Ok((serial.clone(), audio.clone()));
    }

    let modules = discovery::scan_modules().map_err(|e| e.to_string())?;
    let module = modules
        .into_iter()
        .find(|m| !m.audio_device.is_empty())
        .ok_or_else(|| {
            "no EC20 module with audio device found. Use --serial and --audio to specify manually."
                .to_string()
        })?;

    tracing::info!(
        module_id = %module.id,
        usb_serial = %module.usb_serial,
        "auto-discovered module"
    );
    Ok((module.serial_port, module.audio_device))
}

fn probe_module(at: &mut AtCommander) -> Result<(), String> {
    match at.send_command("AT") {
        Ok(AtResponse::Ok(_)) => {
            tracing::info!("AT probe OK");
            Ok(())
        }
        Ok(AtResponse::Error(e)) => Err(format!("AT probe returned ERROR: {e}")),
        Ok(AtResponse::CmeError(code, msg)) => {
            Err(format!("AT probe returned +CME ERROR {code}: {msg}"))
        }
        Err(e) => Err(format!("AT probe failed: {e}")),
    }
}

fn configure_module(at: &mut AtCommander) {
    at.send_command("ATE0").ok();
    at.send_command("AT+CLIP=1").ok();
    route_audio_to_usb(at);
}

fn route_audio_to_usb(at: &mut AtCommander) {
    match at.send_command("AT+QPCMV=1,2") {
        Ok(AtResponse::Ok(_)) => {
            tracing::info!("voice audio routed to USB (AT+QPCMV=1,2)");
        }
        _ => {
            tracing::warn!("AT+QPCMV=1,2 failed, trying AT+QPCMV=1,0");
            match at.send_command("AT+QPCMV=1,0") {
                Ok(AtResponse::Ok(_)) => {
                    tracing::info!("voice audio routed to USB (AT+QPCMV=1,0)");
                }
                _ => {
                    tracing::error!("failed to route voice audio to USB — audio will not work");
                }
            }
        }
    }
}

fn run_event_loop(at: &mut AtCommander) -> ExitCode {
    let mut in_call = false;

    loop {
        let line = match at.read_line_raw() {
            Ok(l) => l,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("timeout") || msg.contains("TimedOut") {
                    continue;
                }
                tracing::error!(error = %msg, "serial read error");
                return ExitCode::from(1);
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        tracing::debug!(urc = trimmed, "received");

        if trimmed == "RING" && !in_call {
            tracing::info!("incoming call (RING), answering...");
            let caller_id = extract_clip(at);
            if !caller_id.is_empty() {
                tracing::info!(caller = %caller_id, "caller ID");
            }

            match at.answer_call() {
                Ok(()) => {
                    in_call = true;
                    tracing::info!("call answered — audio loopback active");
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to answer call");
                }
            }
        } else if (trimmed == "NO CARRIER" || trimmed == "BUSY" || trimmed == "NO ANSWER")
            && in_call
        {
            in_call = false;
            tracing::info!("call ended ({trimmed}), returning to idle");
        }
    }
}

fn extract_clip(at: &mut AtCommander) -> String {
    for _ in 0..5 {
        match at.read_line_raw() {
            Ok(line) => {
                let trimmed = line.trim();
                if let Some(clip_data) = trimmed.strip_prefix("+CLIP:") {
                    if let Some(number) = clip_data.split(',').next() {
                        return number.trim().trim_matches('"').to_string();
                    }
                }
                if trimmed == "RING" || trimmed.is_empty() {
                    continue;
                }
            }
            Err(_) => break,
        }
    }
    String::new()
}
