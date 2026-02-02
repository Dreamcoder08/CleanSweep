// AI-First Protocol: Output estructurado para integración con Claude y otras AIs

use dreamcoder_core::{Operation, OperationResult, Status};
use serde::{Deserialize, Serialize};

/// Protocolo de comunicación AI-First
/// Toda la salida debe ser parseable por máquinas
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AiResponse {
    pub version: String,
    pub timestamp: String,
    pub request_id: String,
    pub status: AiStatus,
    pub data: AiData,
    pub requires_action: Option<AiAction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum AiStatus {
    Success,
    Partial,
    Failed,
    RequiresInput,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum AiData {
    Operations(Vec<Operation>),
    Modules(Vec<dreamcoder_core::Module>),
    State(serde_json::Value),
    Error { code: String, message: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AiAction {
    pub action_type: ActionType,
    pub message: String,
    pub options: Option<Vec<String>>,
    pub default: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ActionType {
    Confirm,
    Input,
    Select,
    Password,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Confirm => write!(f, "confirm"),
            ActionType::Input => write!(f, "input"),
            ActionType::Select => write!(f, "select"),
            ActionType::Password => write!(f, "password"),
        }
    }
}

#[allow(dead_code)]
impl AiResponse {
    pub fn from_result(result: OperationResult) -> Self {
        let has_pending = result
            .operations
            .iter()
            .any(|o| matches!(o.status, Status::PendingInput));

        let all_success = result
            .operations
            .iter()
            .all(|o| matches!(o.status, Status::Success));

        let status = if has_pending {
            AiStatus::RequiresInput
        } else if all_success {
            AiStatus::Success
        } else if result.success {
            AiStatus::Partial
        } else {
            AiStatus::Failed
        };

        let requires_action = result
            .operations
            .iter()
            .find(|o| matches!(o.status, Status::PendingInput))
            .and_then(|o| o.requires_input.as_ref())
            .map(|input| AiAction {
                action_type: match input.input_type {
                    dreamcoder_core::InputType::Confirm => ActionType::Confirm,
                    dreamcoder_core::InputType::Text => ActionType::Input,
                    dreamcoder_core::InputType::Password => ActionType::Password,
                    dreamcoder_core::InputType::Select => ActionType::Select,
                },
                message: input.message.clone(),
                options: None, // TODO: Parse select options
                default: input.default.clone(),
            });

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: uuid::Uuid::new_v4().to_string(),
            status,
            data: AiData::Operations(result.operations),
            requires_action,
        }
    }

    pub fn write_json(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        println!("{}", json);
        Ok(())
    }
}

/// Protocolo de entrada para AI
/// Claude puede enviar comandos estructurados
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AiRequest {
    pub command: String,
    pub args: Vec<String>,
    pub context: AiContext,
    pub auto_confirm: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AiContext {
    pub workspace: String,
    pub os: String,
    pub arch: String,
}

// Ejemplo de uso con Claude:
//
// 1. Claude ejecuta: `dreamcoder status --json`
// 2. Recibe estado actual parseable
// 3. Detecta cambios necesarios
// 4. Ejecuta: `dreamcoder apply --json --yes` (modo auto)
// 5. Recibe resultado estructurado
// 6. Si requiere input, Claude puede responder o pedirte a vos
//
// Ejemplo output:
// ```json
// {
//   "version": "2.0.0",
//   "timestamp": "2025-02-02T15:30:00Z",
//   "status": "requires_input",
//   "data": {
//     "operations": [
//       {
//         "id": "install-shell",
//         "action": "install",
//         "status": "pending_input",
//         "requires_input": {
//           "type": "confirm",
//           "message": "Overwrite ~/.zshrc?",
//           "default": "false"
//         }
//       }
//     ]
//   },
//   "requires_action": {
//     "type": "confirm",
//     "message": "Overwrite ~/.zshrc?",
//     "default": "false"
//   }
// }
// ```
//
// Claude puede entender esto y:
// - Si auto_confirm=true y default=true: enviar confirmación automática
// - Si hay conflicto: preguntarte "¿Sobreescribir ~/.zshrc? (S/n)"
