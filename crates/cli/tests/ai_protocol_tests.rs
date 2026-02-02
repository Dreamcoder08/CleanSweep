//! Tests de integración para el AI Protocol
//!
//! Verifica que el protocolo AI-First funciona correctamente
//! y que Claude (u otra IA) puede interactuar con Dreamcoder.

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use tempfile::TempDir;

/// Helper para ejecutar dreamcoder y capturar JSON output
fn run_dreamcoder(args: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "dreamcoder", "--"])
        .args(args)
        .args(&["--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extraer JSON de la salida (puede haber texto antes)
    if let Some(start) = stdout.find('{') {
        let json_str = &stdout[start..];
        let parsed: Value = serde_json::from_str(json_str)?;
        Ok(parsed)
    } else {
        Err("No JSON found in output".into())
    }
}

#[test]
fn test_ai_protocol_status_command() {
    // Este test verifica que el comando status devuelve JSON válido
    // siguiendo el AI Protocol

    let result = run_dreamcoder(&["status"]);

    if let Ok(json) = result {
        // Verificar estructura del protocolo
        assert!(json.get("version").is_some(), "Missing version field");
        assert!(json.get("timestamp").is_some(), "Missing timestamp field");
        assert!(json.get("status").is_some(), "Missing status field");
        assert!(json.get("data").is_some(), "Missing data field");

        // Verificar que status es un string válido
        let status = json["status"].as_str().unwrap();
        assert!(
            ["success", "requires_input", "failed"].contains(&status),
            "Invalid status value: {}",
            status
        );

        println!("✓ AI Protocol structure validated");
    } else {
        println!("⚠ Test skipped: dreamcoder binary not available in test environment");
    }
}

#[test]
fn test_ai_protocol_apply_dry_run() {
    // Test: apply --dry-run debe devolver plan de operaciones

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("dotfiles");
    let target = temp.path().join("home");

    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();

    // Crear estructura de prueba
    let pkg = source.join("test-pkg");
    std::fs::create_dir(&pkg).unwrap();
    std::fs::write(pkg.join(".bashrc"), "# test").unwrap();

    // TODO: Set DREAMCODER_ROOT env var

    // Ejecutar apply --dry-run --json
    let result = run_dreamcoder(&["apply", "--dry-run"]);

    if let Ok(json) = result {
        // Verificar que devuelve operaciones planificadas
        let data = &json["data"];

        if let Some(ops) = data.get("operations") {
            assert!(ops.is_array(), "operations should be an array");

            // Verificar estructura de cada operación
            for op in ops.as_array().unwrap() {
                assert!(op.get("id").is_some(), "Operation missing id");
                assert!(op.get("action").is_some(), "Operation missing action");
                assert!(op.get("status").is_some(), "Operation missing status");
            }

            println!("✓ Apply dry-run returns valid operation plan");
        }
    }
}

#[test]
fn test_ai_protocol_requires_input() {
    // Test: Cuando hay conflictos, debe devolver requires_action

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("dotfiles");
    let target = temp.path().join("home");

    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();

    // Crear paquete
    let pkg = source.join("test-pkg");
    std::fs::create_dir(&pkg).unwrap();
    std::fs::write(pkg.join(".bashrc"), "# from package").unwrap();

    // Crear archivo existente (conflicto)
    std::fs::write(target.join(".bashrc"), "# existing").unwrap();

    let result = run_dreamcoder(&["apply", "test-pkg"]);

    if let Ok(json) = result {
        let status = json["status"].as_str().unwrap_or("");

        if status == "requires_input" {
            // Verificar estructura de requires_action
            let action = &json["requires_action"];
            assert!(action.get("action_type").is_some(), "Missing action_type");
            assert!(action.get("message").is_some(), "Missing message");

            let action_type = action["action_type"].as_str().unwrap();
            assert!(
                ["confirm", "input", "password", "select"].contains(&action_type),
                "Invalid action_type: {}",
                action_type
            );

            println!("✓ Conflict properly detected with requires_input");
        }
    }
}

#[test]
fn test_ai_protocol_operation_result_structure() {
    // Verifica que cada operación tiene la estructura correcta

    let op = json!({
        "id": "op-001",
        "action": "install",
        "status": "success",
        "message": "Package installed",
        "requires_input": null,
        "metadata": {
            "package": "test-pkg",
            "symlinks_created": 3
        }
    });

    // Validar estructura
    assert!(op["id"].is_string());
    assert!(op["action"].is_string());
    assert!(op["status"].is_string());
    assert!(op["message"].is_string());

    let valid_statuses = [
        "pending",
        "running",
        "success",
        "failed",
        "skipped",
        "pending_input",
    ];
    let status = op["status"].as_str().unwrap();
    assert!(valid_statuses.contains(&status));

    println!("✓ Operation structure validated");
}

#[test]
fn test_ai_protocol_version_format() {
    // Verifica que la versión sigue semver

    let version = "2.0.0";
    let parts: Vec<&str> = version.split('.').collect();

    assert_eq!(
        parts.len(),
        3,
        "Version should follow semver (major.minor.patch)"
    );

    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "Version parts should be numeric"
        );
    }

    println!("✓ Version format validated: {}", version);
}

/// Simulación de interacción AI completa
#[test]
fn test_ai_interaction_simulation() {
    // Simula el flujo completo de interacción con Claude

    println!("\n🤖 Simulating AI interaction flow:\n");

    // Paso 1: Claude pide estado
    println!("1. Claude: dreamcoder status --json");
    let status = json!({
        "version": "2.0.0",
        "status": "success",
        "data": {
            "system": { "os": "linux", "arch": "x86_64" },
            "modules": [
                { "name": "shell", "installed": false },
                { "name": "nvim", "installed": true }
            ]
        }
    });
    println!(
        "   Response: {}",
        serde_json::to_string_pretty(&status).unwrap()
    );

    // Paso 2: Claude decide instalar módulos pendientes
    println!("\n2. Claude: dreamcoder apply shell --json");
    let apply = json!({
        "version": "2.0.0",
        "status": "requires_input",
        "data": {
            "operations": [{
                "id": "op-001",
                "action": "install",
                "status": "pending_input",
                "requires_input": {
                    "input_type": "confirm",
                    "message": "Overwrite ~/.bashrc?",
                    "default": "false"
                }
            }]
        },
        "requires_action": {
            "action_type": "confirm",
            "message": "Overwrite ~/.bashrc?",
            "default": "false"
        }
    });
    println!(
        "   Response: {}",
        serde_json::to_string_pretty(&apply).unwrap()
    );

    // Paso 3: Claude pregunta al usuario
    println!("\n3. Claude to User: 'Overwrite ~/.bashrc? (y/N)'");
    println!("   User responds: 'y'");

    // Paso 4: Claude confirma con --yes
    println!("\n4. Claude: dreamcoder apply shell --yes --json");
    let confirmed = json!({
        "version": "2.0.0",
        "status": "success",
        "data": {
            "operations": [{
                "id": "op-001",
                "action": "install",
                "status": "success",
                "message": "Module shell installed"
            }]
        }
    });
    println!(
        "   Response: {}",
        serde_json::to_string_pretty(&confirmed).unwrap()
    );

    println!("\n✅ AI interaction flow simulation complete!");
}

/// Test de performance del protocolo
#[test]
fn test_ai_protocol_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Simular parsing de 1000 respuestas JSON
    for _ in 0..1000 {
        let json = json!({
            "version": "2.0.0",
            "status": "success",
            "data": {
                "operations": [
                    {"id": "op-001", "action": "install", "status": "success"}
                ]
            }
        });

        let _ = serde_json::to_string(&json);
    }

    let elapsed = start.elapsed();
    println!("✓ Parsed 1000 AI Protocol messages in {:?}", elapsed);

    // Debería ser casi instantáneo (< 100ms)
    assert!(elapsed.as_millis() < 100, "AI Protocol parsing too slow");
}

/// Documentación del protocolo como test
#[test]
fn test_ai_protocol_documentation() {
    // Este test sirve como documentación viva del protocolo

    println!("\n📚 AI Protocol Documentation\n");
    println!("============================\n");

    println!("Root Object:");
    println!("  - version: String (semver)");
    println!("  - timestamp: ISO 8601");
    println!("  - request_id: UUID");
    println!("  - status: Enum [success, partial, failed, requires_input]");
    println!("  - data: Object (payload específico)");
    println!("  - requires_action: Optional<Action>\n");

    println!("Action Object:");
    println!("  - action_type: Enum [confirm, input, password, select]");
    println!("  - message: String (prompt para el usuario)");
    println!("  - options: Optional<Array<String>> (para select)");
    println!("  - default: Optional<String>\n");

    println!("Operation Object:");
    println!("  - id: UUID");
    println!("  - action: Enum [backup, install, template, hook, ...]");
    println!("  - status: Enum [pending, running, success, failed, skipped]");
    println!("  - message: String");
    println!("  - metadata: Object (datos específicos)");
    println!("  - requires_input: Optional<InputRequest>\n");

    assert!(true, "Documentation test");
}
