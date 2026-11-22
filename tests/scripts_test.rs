//! Test ripetibili per gli script `trackerUp` e `trackerDown`.
//!
//! Verifica che:
//! - `trackerUp` avvii il server e salvi il PID
//! - il processo resti vivo dopo l'avvio
//! - `trackerDown` fermi il server e rimuova il PID file

use std::process::Command;

const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");
const BUILDS_DIR: &str = "builds";
const PID_FILE: &str = "builds/nxm-tracker.pid";

fn run_script(name: &str) -> std::process::Output {
    Command::new(format!("{PROJECT_DIR}/scripts/{name}"))
        .current_dir(PROJECT_DIR)
        .output()
        .expect("impossibile eseguire lo script {name}")
}

fn is_process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn read_pid() -> Option<u32> {
    let path = format!("{PROJECT_DIR}/{PID_FILE}");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn ensure_binary_built() {
    let binary = format!("{PROJECT_DIR}/{BUILDS_DIR}/nxm-tracker");
    if !std::path::Path::new(&binary).exists() {
        let _ = Command::new("sh")
            .arg(format!("{PROJECT_DIR}/scripts/build.sh"))
            .current_dir(PROJECT_DIR)
            .status();
    }
}

#[test]
fn tracker_up_starts_server_and_creates_pid_file() {
    // Pulizia eventuale di esecuzioni precedenti.
    let _ = run_script("trackerDown");

    ensure_binary_built();

    let output = run_script("trackerUp");
    assert!(
        output.status.success(),
        "trackerUp dovrebbe uscire con codice 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Il PID file deve esistere.
    let pid = read_pid().expect("trackerUp deve creare builds/nxm-tracker.pid");
    assert!(pid > 0, "PID deve essere maggiore di 0, ottenuto {pid}");

    // Il processo deve essere vivo.
    assert!(
        is_process_alive(pid),
        "il processo PID {pid} dovrebbe essere vivo dopo trackerUp"
    );

    // Pulizia: ferma il server.
    let down = run_script("trackerDown");
    assert!(
        down.status.success(),
        "trackerDown dovrebbe uscire con codice 0, stderr: {}",
        String::from_utf8_lossy(&down.stderr)
    );

    // Il processo non deve più essere vivo.
    assert!(
        !is_process_alive(pid),
        "il processo PID {pid} non dovrebbe essere vivo dopo trackerDown"
    );

    // Il PID file deve essere rimosso.
    assert!(
        !std::path::Path::new(&format!("{PROJECT_DIR}/{PID_FILE}")).exists(),
        "trackerDown deve rimuovere builds/nxm-tracker.pid"
    );
}

#[test]
fn tracker_down_without_pid_file_is_not_an_error() {
    // Rimuovi eventuali residui.
    let _ = run_script("trackerDown");
    let pid_path = format!("{PROJECT_DIR}/{PID_FILE}");
    if std::path::Path::new(&pid_path).exists() {
        std::fs::remove_file(&pid_path).expect("rimozione pid file");
    }

    let output = run_script("trackerDown");
    assert!(
        output.status.success(),
        "trackerDown senza PID file non dovrebbe fallire, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tracker_up_when_already_running_is_idempotent() {
    let _ = run_script("trackerDown");
    ensure_binary_built();

    let first = run_script("trackerUp");
    assert!(first.status.success(), "primo trackerUp dovrebbe riuscire");

    let pid1 = read_pid().expect("PID file dopo primo trackerUp");

    // Secondo trackerUp: deve rilevare che è già attivo e uscire con 0.
    let second = run_script("trackerUp");
    assert!(
        second.status.success(),
        "secondo trackerUp dovrebbe uscire con 0 (già attivo)"
    );

    let pid2 = read_pid().expect("PID file dopo secondo trackerUp");
    assert_eq!(pid1, pid2, "lo stesso PID deve essere mantenuto");

    // Pulizia.
    let _ = run_script("trackerDown");
}
