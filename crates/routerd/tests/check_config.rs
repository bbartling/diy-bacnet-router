use std::process::Command;

#[test]
fn check_config_succeeds_for_example_toml() {
    let example = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/router.example.toml"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_diy-bacnet-router"))
        .args(["--check-config", "--config", example])
        .output()
        .expect("run check-config");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("configuration OK"));
}

#[test]
fn check_config_fails_for_duplicate_networks() {
    let dir = std::env::temp_dir().join(format!("dbr-bad-config-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp");
    let path = dir.join("bad.toml");
    std::fs::write(
        &path,
        r#"
[identity]
name = "x"
location = "y"
[management]
bind = "127.0.0.1:8080"
web_root = "frontend/web/dist"
metrics_interval_ms = 1000
max_ws_connections = 8
[router]
enabled = false
[bacnet_ip]
interface = "eth0"
bind_address = "0.0.0.0"
udp_port = 47808
network = 1
bbmd_enabled = false
foreign_device_enabled = false
[mstp]
serial = "/dev/serial/by-id/x"
adapter_profile = "waveshare-usb-to-rs485-c"
termination = "onboard-present"
baud = 38400
mac = 3
network = 1
max_master = 127
max_info_frames = 1
"#,
    )
    .expect("write invalid");

    let output = Command::new(env!("CARGO_BIN_EXE_diy-bacnet-router"))
        .args(["--check-config", "--config", path.to_str().unwrap()])
        .output()
        .expect("run check-config");
    assert!(!output.status.success());
    let _ = std::fs::remove_dir_all(dir);
}
