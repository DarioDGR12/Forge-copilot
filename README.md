# Forge Copilot

Asistente de escritorio para **Pop!_OS** (y otros Linux): overlay tipo Windows Copilot, chat en streaming estilo Claude, **BYOK** y herramientas locales (terminal, archivos, apps, procesos, captura de pantalla). Las acciones peligrosas piden confirmación.

Este repositorio es la app que corre **en tu máquina**. No es un servicio en la nube y no controla otros equipos.

## Qué incluye (v1)

- Panel lateral (siempre encima) con atajo `Ctrl+Shift+Space`
- Bandeja del sistema: mostrar/ocultar y salir (el cierre oculta, no mata el proceso)
- Chat con Markdown, historial SQLite y bloques de herramientas
- Proveedores: demostración, OpenAI, Anthropic, OpenRouter, Ollama
- Claves en el keyring de Linux (`secret-service`); si no hay llavero, archivo `0600` en `~/.local/share/forge-copilot/secrets.json`
- Tools: `run_terminal`, `read_file`, `write_file`, `list_dir`, `list_apps`, `list_processes`, `screenshot`, `host_info`, `launch_app`, `open_path`, `clipboard_read`, `clipboard_write`, `notify`, `list_windows`, `focus_window`
- Adjuntar un archivo de texto al mensaje (32 KB) y barra de estado con proveedor, modelo y atajo
- Política: lecturas, portapapeles (lectura), ventanas, notificaciones y abrir rutas dentro de `$HOME` en automático; shell, lanzar apps, escribir archivos o portapapeles, enfocar ventanas y rutas sensibles piden Allow/Deny

## Requisitos en Pop!_OS

- Node.js 20+ y Rust estable
- Dependencias Tauri 2:

```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf \
  libgtk-3-dev libayatana-appindicator3-dev
```

Herramientas de escritorio opcionales:

```bash
sudo apt install grim wl-clipboard xclip libnotify-bin wmctrl
```

En COSMIC/Wayland `wmctrl` a menudo no lista ventanas: Forge lo dice, no simula clics. Portapapeles usa `wl-copy`/`wl-paste` o `xclip`/`xsel`.

## Desarrollo

```bash
npm install
npm run tauri dev
```

La UI también corre en el navegador (herramientas **simuladas**):

```bash
npm run dev
```

Tests de política y parsers:

```bash
npm test
```

## Empaquetar `.deb`

```bash
npm run tauri build
```

El paquete queda en `src-tauri/target/release/bundle/deb/`. Instala con:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/forge-copilot_0.1.0_*.deb
```

También se genera AppImage si el entorno de build lo permite.

## Uso

1. Abre Forge Copilot (o el icono de bandeja).
2. En **Ajustes** elige proveedor y, si no es demo/Ollama, pega tu API key → **Guardar clave**.
3. Habla en el chat. Si Forge quiere ejecutar un comando o escribir un archivo, confirma en el modal.
4. Atajo: `Ctrl+Shift+Space`. En COSMIC/GNOME, Super está reservado.

### Wayland

El atajo global y el anclaje exacto al borde derecho pueden fallar según el compositor. Usa la bandeja como respaldo. La captura usa `grim`, `gnome-screenshot`, `spectacle` o `import` si existen.

### Flatpak

Queda fuera de v1: el sandbox pelea con un agente que necesita terminal y sistema de archivos reales.

## Privacidad

- Historial en `~/.local/share/forge-copilot/forge.db`
- Las keys no se mandan a ningún servidor de Forge (no hay backend propio)
- El modelo que elijas (OpenAI, Anthropic, etc.) sí recibe el texto del chat y los resultados de tools que se le reenvían

## Licencia

Unlicense (dominio público). Ver `LICENSE`.
