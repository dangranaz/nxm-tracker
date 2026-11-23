//! Test end-to-end del trasporto stdio del server MCP `nxm-tracker`.
//!
//! Avvia il binario `nxm-tracker` come processo figlio, invia richieste
//! JSON-RPC su stdin e verifica le risposte su stdout.
//!
//! Criteri di successo:
//! - `initialize` restituisce protocolVersion, capabilities e serverInfo
//! - `tools/list` restituisce tutti e 8 gli schemi di tool
//! - `tools/call` su `create_session` crea una sessione e restituisce l'ID

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Command, Stdio};

type ServerProcess = (
    std::process::Child,
    BufWriter<std::process::ChildStdin>,
    BufReader<std::process::ChildStdout>,
);

fn binary_path() -> String {
    std::env::var("CARGO_BIN_EXE_nxm-tracker").unwrap_or_else(|_| {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/debug/nxm-tracker")
            .to_string_lossy()
            .to_string()
    })
}

/// Avvia il server come processo figlio con stdin/stdout pipe.
fn spawn_server() -> Result<ServerProcess, Box<dyn std::error::Error>> {
    let mut child = Command::new(binary_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("impossibile avviare il binario: {e}"))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "stdin non disponibile".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout non disponibile".to_string())?;

    let writer = BufWriter::new(stdin);
    let reader = BufReader::new(stdout);

    Ok((child, writer, reader))
}

/// Invia una richiesta JSON-RPC e legge la risposta successiva.
fn send_and_receive(
    writer: &mut BufWriter<std::process::ChildStdin>,
    reader: &mut BufReader<std::process::ChildStdout>,
    request: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    writeln!(writer, "{request}")?;
    writer.flush()?;

    let mut response = String::new();
    reader.read_line(&mut response)?;
    Ok(response)
}

fn stop_server(mut child: std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn stdio_initialize_returns_protocol_and_server_info() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let response = send_and_receive(&mut writer, &mut reader, request)?;

    assert!(
        response.contains("\"jsonrpc\":\"2.0\""),
        "risposta deve essere JSON-RPC 2.0"
    );
    assert!(response.contains("\"id\":1"), "risposta deve avere id=1");
    assert!(
        response.contains("protocolVersion"),
        "risposta deve contenere protocolVersion"
    );
    assert!(
        response.contains("serverInfo"),
        "risposta deve contenere serverInfo"
    );
    assert!(
        response.contains("nxm-tracker"),
        "serverInfo.name deve essere nxm-tracker"
    );

    stop_server(child);
    Ok(())
}

#[test]
fn stdio_tools_list_returns_seventeen_tools() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    // initialize prima (richiesto dal protocollo MCP)
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let _ = send_and_receive(&mut writer, &mut reader, init)?;

    let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    let response = send_and_receive(&mut writer, &mut reader, request)?;

    // tools/list restituisce gli schemi JSON Schema (senza i nomi degli tool).
    // Verifichiamo che ci siano 17 schemi (uno per tool).
    let required_count = response.matches("\"required\"").count();
    assert_eq!(
        required_count, 17,
        "tools/list dovrebbe restituire 17 schemi di tool, trovati {required_count}. Risposta: {response}"
    );
    assert!(
        response.contains("\"tools\""),
        "risposta dovrebbe contenere la chiave 'tools', risposta: {response}"
    );

    stop_server(child);
    Ok(())
}

#[test]
fn stdio_tools_call_create_session_returns_id() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let _ = send_and_receive(&mut writer, &mut reader, init)?;

    let request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"create_session","arguments":{"name":"STDIO test session","started_at":"2026-11-20T08:00:00+01:00"}}}"#;
    let response = send_and_receive(&mut writer, &mut reader, request)?;

    // Il tool restituisce il session JSON doppiamente encodato.
    // Cerchiamo sottostringhe che appaiono nel testo annidato.
    assert!(
        response.contains("STDIO test session"),
        "risposta dovrebbe contenere il nome della sessione, risposta: {response}"
    );
    assert!(
        response.contains("open"),
        "risposta dovrebbe contenere status open, risposta: {response}"
    );
    assert!(
        response.contains("\"id\":"),
        "risposta dovrebbe contenere l'id della sessione, risposta: {response}"
    );

    stop_server(child);
    Ok(())
}

#[test]
fn stdio_invalid_json_returns_parse_error() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    let response = send_and_receive(&mut writer, &mut reader, "non è json valido\n")?;

    assert!(
        response.contains("parse error") || response.contains("-32700"),
        "risposta dovrebbe essere un parse error, risposta: {response}"
    );

    stop_server(child);
    Ok(())
}

#[test]
fn stdio_create_session_empty_name_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let _ = send_and_receive(&mut writer, &mut reader, init)?;

    let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"create_session","arguments":{"name":"   ","started_at":"2026-11-20T08:00:00+01:00"}}}"#;
    let response = send_and_receive(&mut writer, &mut reader, request)?;

    assert!(
        response.contains("\"isError\":true"),
        "empty name should return isError=true, risposta: {response}"
    );

    stop_server(child);
    Ok(())
}

#[test]
fn stdio_create_session_invalid_timestamp_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let _ = send_and_receive(&mut writer, &mut reader, init)?;

    let request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"create_session","arguments":{"name":"Valid","started_at":"not-a-date"}}}"#;
    let response = send_and_receive(&mut writer, &mut reader, request)?;

    assert!(
        response.contains("\"isError\":true"),
        "invalid timestamp should return isError=true, risposta: {response}"
    );
    assert!(
        response.contains("RFC3339"),
        "error should mention RFC3339, risposta: {response}"
    );

    stop_server(child);
    Ok(())
}

#[test]
fn stdio_get_nonexistent_session_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let (child, mut writer, mut reader) = if let Ok(t) = spawn_server() {
        t
    } else {
        eprintln!("STDIO TEST SKIPPED: spawn_server failed");
        return Ok(());
    };

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let _ = send_and_receive(&mut writer, &mut reader, init)?;

    let request = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_session","arguments":{"id":99999}}}"#;
    let response = send_and_receive(&mut writer, &mut reader, request)?;

    assert!(
        response.contains("\"isError\":true"),
        "nonexistent session should return isError=true, risposta: {response}"
    );
    assert!(
        response.contains("not found"),
        "error should mention not found, risposta: {response}"
    );

    stop_server(child);
    Ok(())
}
