use handlebars::{Handlebars, Renderable};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    context: TemplateContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    pub system: SystemInfo,
    pub user: UserInfo,
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub home: PathBuf,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // Register helpers
        Self::register_helpers(&mut handlebars);

        let context = TemplateContext {
            system: SystemInfo {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                hostname: hostname::get()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_default(),
            },
            user: UserInfo {
                name: std::env::var("USER").unwrap_or_default(),
                email: String::new(), // From config
                home: dirs::home_dir().unwrap_or_default(),
            },
            custom: HashMap::new(),
        };

        Self {
            handlebars,
            context,
        }
    }

    pub fn with_config(_config: &dreamcoder_core::Config) -> Self {
        Self::new()
    }

    fn register_helpers(handlebars: &mut Handlebars) {
        use handlebars::{
            Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError,
        };

        // Helper condicional por OS
        struct IfOsHelper;
        impl HelperDef for IfOsHelper {
            fn call<'reg: 'rc, 'rc>(
                &self,
                h: &Helper<'rc>,
                hb: &'reg Handlebars<'reg>,
                ctx: &'rc Context,
                rctx: &mut RenderContext<'reg, 'rc>,
                out: &mut dyn Output,
            ) -> Result<(), RenderError> {
                let os = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
                let current = std::env::consts::OS;

                if os == current {
                    if let Some(t) = h.template() {
                        t.render(hb, ctx, rctx, out)?;
                    }
                }
                Ok(())
            }
        }

        // Helper para archivos existentes
        struct IfExistsHelper;
        impl HelperDef for IfExistsHelper {
            fn call<'reg: 'rc, 'rc>(
                &self,
                h: &Helper<'rc>,
                hb: &'reg Handlebars<'reg>,
                ctx: &'rc Context,
                rctx: &mut RenderContext<'reg, 'rc>,
                out: &mut dyn Output,
            ) -> Result<(), RenderError> {
                let path = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
                let expanded = shellexpand::tilde(path);

                if Path::new(expanded.as_ref()).exists() {
                    if let Some(t) = h.template() {
                        t.render(hb, ctx, rctx, out)?;
                    }
                }
                Ok(())
            }
        }

        handlebars.register_helper("if_os", Box::new(IfOsHelper));
        handlebars.register_helper("if_exists", Box::new(IfExistsHelper));
    }

    pub fn render(&self, template: &str) -> Result<String, TemplateError> {
        let data = json!({
            "system": self.context.system,
            "user": self.context.user,
            "custom": self.context.custom,
        });

        self.handlebars
            .render_template(template, &data)
            .map_err(|e| TemplateError::Render(e.to_string()))
    }

    pub fn render_file(&self, path: &Path) -> Result<String, TemplateError> {
        let content = std::fs::read_to_string(path).map_err(|e| TemplateError::Io(e))?;
        self.render(&content)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template render error: {0}")]
    Render(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Ejemplo de template:
// # ~/.gitconfig
// [user]
// name = {{user.name}}
// email = {{user.email}}
//
// {{#if_os "linux"}}
// [core]
// editor = nvim
// {{/if_os}}
//
// {{#if_os "macos"}}
// [core]
// editor = code --wait
// {{/if_os}}
//
// {{#if_exists "~/.config/custom"}}
// [include]
// path = ~/.config/custom
// {{/if_exists}}
