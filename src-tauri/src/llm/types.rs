use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Assistant,
    Tool,
    System,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::System => "system",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "assistant" => Self::Assistant,
            "tool" => Self::Tool,
            "system" => Self::System,
            _ => Self::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: Role,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Default)]
pub struct AssistantTurn {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
}

pub const SYSTEM_PROMPT: &str = "\
Eres Forge Copilot, un asistente de escritorio local para Pop!_OS y Linux.
Ayudas con la terminal, archivos, aplicaciones, procesos, capturas, portapapeles, ventanas, notificaciones y entrada (teclado/ratón).
Responde en el idioma del usuario (por defecto español). Sé claro y conciso.
Usa herramientas cuando necesites datos reales del sistema; no inventes salidas de comandos.
Puedes listar/abrir archivos, lanzar apps .desktop, ver procesos, capturar la pantalla, leer/escribir el portapapeles, listar/enfocar ventanas (wmctrl), enviar notificaciones, teclear, pulsar atajos y hacer clic.
Si hay una captura reciente, descríbela con lo que se ve; no inventes iconos ni texto ilegible.
Teclear/clic oculta el overlay para no robar el foco. En COSMIC/Wayland hace falta ydotool (o xdotool en X11); si faltan, explica el error — no simules clics.
Explica en una frase qué vas a hacer antes o después de una herramienta.
No intentes atacar otras máquinas, no hagas phishing ni uses la red de forma ofensiva.
Las acciones peligrosas las confirmará el usuario en la interfaz.";

pub fn tool_specs() -> serde_json::Value {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "run_terminal",
                "description": "Ejecuta un comando en bash en el sistema local del usuario.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Comando bash" },
                        "cwd": { "type": "string", "description": "Directorio de trabajo opcional" }
                    },
                    "required": ["command"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Lee un archivo de texto del sistema de archivos local.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Escribe o crea un archivo de texto local.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_dir",
                "description": "Lista entradas de un directorio local.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_apps",
                "description": "Lista aplicaciones de escritorio instaladas (.desktop).",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_processes",
                "description": "Lista procesos en ejecución ordenados por CPU.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer" }
                    }
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "screenshot",
                "description": "Captura la pantalla actual y guarda un PNG temporal.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "host_info",
                "description": "Devuelve usuario, hostname, distro y escritorio de esta máquina.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "launch_app",
                "description": "Lanza una aplicación instalada resolviéndola por nombre .desktop. No ejecuta binarios arbitrarios.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Nombre visible, por ejemplo Firefox" }
                    },
                    "required": ["name"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "open_path",
                "description": "Abre un archivo o carpeta con la app predeterminada (xdg-open).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "clipboard_read",
                "description": "Lee el texto del portapapeles (wl-paste, xclip o xsel).",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "clipboard_write",
                "description": "Escribe texto en el portapapeles. Pide confirmación.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string" }
                    },
                    "required": ["text"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "notify",
                "description": "Muestra una notificación de escritorio con notify-send.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "body": { "type": "string" }
                    },
                    "required": ["title"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_windows",
                "description": "Lista ventanas visibles con wmctrl. En Wayland puro puede no estar disponible.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "focus_window",
                "description": "Enfoca una ventana por título o id (wmctrl). Pide confirmación.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Título o id de ventana" }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "type_text",
                "description": "Teclea texto en la ventana activa (ydotool, xdotool o wtype). Forge se oculta antes. Pide confirmación.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string" }
                    },
                    "required": ["text"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "press_keys",
                "description": "Pulsa un atajo (ctrl+c, Return, alt+Tab). Solo teclas de una lista blanca. Pide confirmación.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "keys": { "type": "string", "description": "Atajo, por ejemplo ctrl+c o Return" }
                    },
                    "required": ["keys"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "mouse_click",
                "description": "Clic izquierdo/derecho/central. Coordenadas opcionales. Pide confirmación.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer" },
                        "y": { "type": "integer" },
                        "button": { "type": "string", "description": "left, right o middle" }
                    }
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "pointer_info",
                "description": "Posición del puntero (xdotool). En Wayland puro suele fallar.",
                "parameters": { "type": "object", "properties": {} }
            }
        }
    ])
}

pub fn anthropic_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "run_terminal",
            "description": "Ejecuta un comando en bash en el sistema local del usuario.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "cwd": { "type": "string" }
                },
                "required": ["command"]
            }
        },
        {
            "name": "read_file",
            "description": "Lee un archivo de texto local.",
            "input_schema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }
        },
        {
            "name": "write_file",
            "description": "Escribe un archivo de texto local.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }
        },
        {
            "name": "list_dir",
            "description": "Lista un directorio local.",
            "input_schema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }
        },
        {
            "name": "list_apps",
            "description": "Lista aplicaciones .desktop instaladas.",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "list_processes",
            "description": "Lista procesos por uso de CPU.",
            "input_schema": {
                "type": "object",
                "properties": { "limit": { "type": "integer" } }
            }
        },
        {
            "name": "screenshot",
            "description": "Captura la pantalla actual.",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "host_info",
            "description": "Devuelve usuario, hostname, distro y escritorio.",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "launch_app",
            "description": "Lanza una app instalada por su nombre .desktop.",
            "input_schema": {
                "type": "object",
                "properties": { "name": { "type": "string" } },
                "required": ["name"]
            }
        },
        {
            "name": "open_path",
            "description": "Abre un archivo o carpeta con xdg-open.",
            "input_schema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }
        },
        {
            "name": "clipboard_read",
            "description": "Lee el texto del portapapeles (wl-paste, xclip o xsel).",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "clipboard_write",
            "description": "Escribe texto en el portapapeles. Pide confirmación.",
            "input_schema": {
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"]
            }
        },
        {
            "name": "notify",
            "description": "Muestra una notificación de escritorio con notify-send.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "body": { "type": "string" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "list_windows",
            "description": "Lista ventanas visibles con wmctrl. En Wayland puro puede no estar disponible.",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "focus_window",
            "description": "Enfoca una ventana por título o id (wmctrl). Pide confirmación.",
            "input_schema": {
                "type": "object",
                "properties": { "query": { "type": "string" } },
                "required": ["query"]
            }
        },
        {
            "name": "type_text",
            "description": "Teclea texto en la ventana activa. Pide confirmación.",
            "input_schema": {
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"]
            }
        },
        {
            "name": "press_keys",
            "description": "Pulsa un atajo de teclado de la lista blanca. Pide confirmación.",
            "input_schema": {
                "type": "object",
                "properties": { "keys": { "type": "string" } },
                "required": ["keys"]
            }
        },
        {
            "name": "mouse_click",
            "description": "Clic del ratón. Coordenadas opcionales. Pide confirmación.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "x": { "type": "integer" },
                    "y": { "type": "integer" },
                    "button": { "type": "string" }
                }
            }
        },
        {
            "name": "pointer_info",
            "description": "Posición del puntero.",
            "input_schema": { "type": "object", "properties": {} }
        }
    ])
}
