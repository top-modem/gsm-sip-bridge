use gsm_sip_bridge::modules::beep::BeepGenerator;

#[test]
fn test_beep_generates_sine_at_correct_frequency() {
    let mut gen = BeepGenerator::new();
    gen.start();

    let mut buf = [0i16; 160];
    gen.fill_buffer(&mut buf);

    let non_zero: Vec<_> = buf.iter().filter(|&&s| s != 0).collect();
    assert!(
        !non_zero.is_empty(),
        "buffer should contain non-zero samples"
    );

    let max_amplitude = buf.iter().map(|s| s.unsigned_abs()).max().unwrap();
    assert!(
        max_amplitude > 15000 && max_amplitude < 17000,
        "amplitude should be ~16000, got {max_amplitude}"
    );
}

#[test]
fn test_beep_silence_when_stopped() {
    let mut gen = BeepGenerator::new();
    gen.start();
    gen.stop();

    let mut buf = [0i16; 160];
    gen.fill_buffer(&mut buf);

    assert!(
        buf.iter().all(|&s| s == 0),
        "all samples should be zero when stopped"
    );
}

#[test]
fn test_beep_correct_period() {
    let mut gen = BeepGenerator::new();
    gen.start();

    let mut buf = [0i16; 8000]; // 1 second at 8kHz
    gen.fill_buffer(&mut buf);

    let mut zero_crossings = 0u32;
    for i in 1..buf.len() {
        if (buf[i - 1] >= 0 && buf[i] < 0) || (buf[i - 1] < 0 && buf[i] >= 0) {
            zero_crossings += 1;
        }
    }
    // 400 Hz = 800 zero crossings per second (2 per cycle)
    let expected = 800;
    let tolerance = 10;
    assert!(
        zero_crossings.abs_diff(expected) < tolerance,
        "expected ~{expected} zero crossings, got {zero_crossings}"
    );
}
