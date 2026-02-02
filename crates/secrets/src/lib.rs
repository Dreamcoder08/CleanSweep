use std::path::Path;
use dreamcoder_core::Config;
use async_trait::async_trait;

#[async_trait]
pub trait SecretProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn encrypt(&self, input: &Path, output: &Path) -> Result<(), SecretError>;
    async fn decrypt(&self, input: &Path, output: Option<&Path>) -> Result<String, SecretError>;
}

pub struct SecretManager {
    providers: Vec<Box<dyn SecretProvider>>,
    active_provider: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),
    
    #[error("Encryption failed: {0}")]
    Encryption(String),
    
    #[error("Decryption failed: {0}")]
    Decryption(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("External command failed: {0}")]
    ExternalCommand(String),
}

impl SecretManager {
    pub fn new(_config: &Config) -> Self {
        let providers: Vec<Box<dyn SecretProvider>> = vec![
            Box::new(AgeProvider::new()),
            Box::new(OnePasswordProvider::new()),
            Box::new(BitwardenProvider::new()),
        ];
        
        // Select first available provider as active
        let active_provider = providers.iter()
            .find(|p| p.is_available())
            .map(|p| p.name().to_string())
            .unwrap_or_else(|| "age".to_string());
        
        Self {
            providers,
            active_provider,
        }
    }
    
    fn get_active_provider(&self) -> Option<&dyn SecretProvider> {
        self.providers.iter()
            .find(|p| p.name() == self.active_provider)
            .map(|p| p.as_ref())
    }
    
    pub async fn encrypt(&self, input: &Path, output: &Path) -> Result<(), SecretError> {
        let provider = self.get_active_provider()
            .ok_or_else(|| SecretError::ProviderNotAvailable(self.active_provider.clone()))?;
        provider.encrypt(input, output).await
    }
    
    pub async fn decrypt(&self, input: &Path, output: Option<&Path>) -> Result<String, SecretError> {
        let provider = self.get_active_provider()
            .ok_or_else(|| SecretError::ProviderNotAvailable(self.active_provider.clone()))?;
        provider.decrypt(input, output).await
    }
}

// Age (file encryption) - 100% offline, no cloud
pub struct AgeProvider {
    identity_file: Option<std::path::PathBuf>,
}

impl AgeProvider {
    pub fn new() -> Self {
        Self {
            identity_file: dirs::home_dir().map(|h| h.join(".config").join("dreamcoder").join("age.key")),
        }
    }
}

#[async_trait]
impl SecretProvider for AgeProvider {
    fn name(&self) -> &str {
        "age"
    }
    
    fn is_available(&self) -> bool {
        which::which("age").is_ok()
    }
    
    async fn encrypt(&self, input: &Path, output: &Path) -> Result<(), SecretError> {
        use tokio::process::Command;
        
        let recipients_file = self.identity_file.as_ref()
            .ok_or_else(|| SecretError::Encryption("No identity file".to_string()))?;
        
        let output = Command::new("age")
            .arg("-r")
            .arg(recipients_file)
            .arg("-o")
            .arg(output)
            .arg(input)
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(SecretError::Encryption(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn decrypt(&self, input: &Path, output: Option<&Path>) -> Result<String, SecretError> {
        use tokio::process::Command;
        
        let identity = self.identity_file.as_ref()
            .ok_or_else(|| SecretError::Decryption("No identity file".to_string()))?;
        
        let mut cmd = Command::new("age");
        cmd.arg("-d")
            .arg("-i")
            .arg(identity)
            .arg(input);
        
        if let Some(out) = output {
            cmd.arg("-o").arg(out);
        }
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(SecretError::Decryption(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// 1Password CLI
pub struct OnePasswordProvider;

impl OnePasswordProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl SecretProvider for OnePasswordProvider {
    fn name(&self) -> &str {
        "1password"
    }
    
    fn is_available(&self) -> bool {
        which::which("op").is_ok()
    }
    
    async fn encrypt(&self, _input: &Path, _output: &Path) -> Result<(), SecretError> {
        // 1Password doesn't encrypt files directly, it stores secrets
        // For files, use Age or upload to 1Password as documents
        Err(SecretError::ProviderNotAvailable(
            "1Password stores secrets, not files. Use 'op create item' or combine with age".to_string()
        ))
    }
    
    async fn decrypt(&self, _input: &Path, _output: Option<&Path>) -> Result<String, SecretError> {
        use tokio::process::Command;
        
        // Example: op read "op://Personal/dotfiles/config"
        let output = Command::new("op")
            .args(&["read", "op://Personal/dotfiles/config"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(SecretError::Decryption(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// Bitwarden CLI
pub struct BitwardenProvider;

impl BitwardenProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl SecretProvider for BitwardenProvider {
    fn name(&self) -> &str {
        "bitwarden"
    }
    
    fn is_available(&self) -> bool {
        which::which("bw").is_ok()
    }
    
    async fn encrypt(&self, _input: &Path, _output: &Path) -> Result<(), SecretError> {
        Err(SecretError::ProviderNotAvailable(
            "Bitwarden stores secrets, not files. Use 'bw create item' or combine with age".to_string()
        ))
    }
    
    async fn decrypt(&self, _input: &Path, _output: Option<&Path>) -> Result<String, SecretError> {
        use tokio::process::Command;
        
        let output = Command::new("bw")
            .args(&["get", "item", "dotfiles"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(SecretError::Decryption(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
