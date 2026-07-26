// Минимальный зонд: печатает какие геймпады найдены и какие кнопки/оси жмутся.
// Это НЕ финальная система — только проверка, что железо читается end-to-end.
use gilrs::{Event, EventType, Gilrs};

fn main() {
    let mut gilrs = match Gilrs::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Не удалось инициализировать gilrs: {e}");
            std::process::exit(1);
        }
    };

    println!("=== Найденные геймпады ===");
    let mut any = false;
    for (id, gp) in gilrs.gamepads() {
        any = true;
        println!("  [{id}] {} — connected={}", gp.name(), gp.is_connected());
    }
    if !any {
        println!("  (ничего не найдено — gilrs не видит устройство)");
    }
    println!("\nЖми кнопки/двигай стики. Ctrl+C для выхода.\n");

    loop {
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(btn, _) => println!("[{id}] нажата  {btn:?}"),
                EventType::ButtonReleased(btn, _) => println!("[{id}] отпущена {btn:?}"),
                EventType::AxisChanged(axis, val, _) => {
                    if val.abs() > 0.5 {
                        println!("[{id}] ось {axis:?} = {val:.2}");
                    }
                }
                EventType::Connected => println!("[{id}] ПОДКЛЮЧЁН"),
                EventType::Disconnected => println!("[{id}] ОТКЛЮЧЁН"),
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
