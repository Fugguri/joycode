//! Single-instance: повторный запуск не плодит окна, а разворачивает уже
//! работающий экземпляр. Реализация — через Unix-сокет (Linux/macOS).
//! На Windows single-instance пока no-op (окно просто запустится ещё раз).
use std::path::PathBuf;

/// Путь к управляющему сокету.
pub fn socket_path() -> PathBuf {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("joycode.sock")
}

/// Если другой экземпляр уже запущен — сигналит ему «покажись» и возвращает true
/// (текущему процессу надо тихо выйти).
pub fn signal_existing_and_exit() -> bool {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::net::UnixStream;
        if let Ok(mut s) = UnixStream::connect(socket_path()) {
            let _ = s.write_all(b"show");
            return true;
        }
        false
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Основной экземпляр: слушает сокет и по любому сообщению разворачивает окно.
pub fn start_listener(ctx: egui::Context) {
    #[cfg(unix)]
    {
        use std::io::Read;
        use std::os::unix::net::UnixListener;

        let path = socket_path();
        let _ = std::fs::remove_file(&path); // убрать протухший сокет
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                log::warn!("single-instance: не удалось создать сокет: {e}");
                return;
            }
        };
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if let Ok(mut s) = conn {
                    let mut buf = [0u8; 8];
                    let _ = s.read(&mut buf);
                    log::info!("получен сигнал «показать окно»");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.request_repaint();
                }
            }
        });
    }
    #[cfg(not(unix))]
    {
        let _ = ctx;
    }
}

/// Убрать сокет при выходе.
pub fn cleanup() {
    let _ = std::fs::remove_file(socket_path());
}
