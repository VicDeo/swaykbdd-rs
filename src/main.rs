use futures_lite::{future::block_on, stream::StreamExt};
use log::SetLoggerError;
use log::{error, info};
use std::collections::HashMap;
use swayipc_async::{Connection, EventType, WindowEvent};

#[derive(Debug)]
enum SwayKbddError {
    LoggerError(SetLoggerError),
    SwayIpcError(swayipc_async::Error),
}

impl From<SetLoggerError> for SwayKbddError {
    fn from(value: SetLoggerError) -> Self {
        SwayKbddError::LoggerError(value)
    }
}

impl From<swayipc_async::Error> for SwayKbddError {
    fn from(value: swayipc_async::Error) -> Self {
        SwayKbddError::SwayIpcError(value)
    }
}

fn main() -> Result<(), SwayKbddError> {
    env_logger::try_init()?;

    block_on(async {
        let connection = Connection::new().await?;
        let mut connection2 = Connection::new().await?;

        let mut pid_hashmap = HashMap::new();
        let mut last_w_pid = None;
        let mut event_stream = connection
            .subscribe([EventType::Window, EventType::Input])
            .await?;

        while let Some(event) = event_stream.next().await {
            match event {
                Ok(event) => match event {
                    swayipc_async::Event::Window(w) => {
                        process_window_event(
                            &mut connection2,
                            w,
                            &mut pid_hashmap,
                            &mut last_w_pid,
                        )
                        .await?;
                    }
                    swayipc_async::Event::Input(input) => {
                        // Need to save keyboard layout here
                        if let Some(keyboard_index) = input.input.xkb_active_layout_index {
                            if let Some(w_pid) = last_w_pid {
                                info!("Current keyboard layout set: {}", keyboard_index);
                                pid_hashmap.insert(w_pid, keyboard_index);
                            }
                        }
                    }
                    _ => unreachable!(),
                },
                Err(err) => {
                    error!("Can't get event, {}", err);
                    return Err(err);
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}

async fn process_window_event(
    connection: &mut Connection,
    w: Box<WindowEvent>,
    pid_hashmap: &mut HashMap<i32, i32>,
    last_w_pid: &mut Option<i32>,
) -> Result<(), swayipc_async::Error> {
    if let Some(w_pid) = w.container.pid {
        match w.change {
            swayipc_async::WindowChange::New => {
                info!(
                    "New window found: {}",
                    w.container.name.unwrap_or_else(|| "null".to_owned())
                );
                pid_hashmap.insert(w_pid, 0);
                *last_w_pid = Some(w_pid);
            }
            swayipc_async::WindowChange::Focus => {
                let window_name = w.container.name.unwrap_or_else(|| "null".to_owned());
                info!("Window focused: {}", &window_name);
                if let Some(saved_keyboard_index) = pid_hashmap.get(&w_pid) {
                    connection
                        .run_command(format!(
                            "input * xkb_switch_layout {}",
                            saved_keyboard_index
                        ))
                        .await?;
                    info!(
                        "Window {} keyboard layout set to {}",
                        &window_name, saved_keyboard_index
                    );
                } else {
                    pid_hashmap.insert(w_pid, 0);
                }
                *last_w_pid = Some(w_pid);
            }
            _ => (),
        }
    }
    Ok(())
}
