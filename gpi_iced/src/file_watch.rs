use std::ffi::OsStr;
use std::path::Component;
use std::time::Duration;

use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::stream;
use iced::Subscription;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use std::path::Path;

use crate::app::Message;

fn file_watcher() -> impl Stream<Item = Message> {
    stream::channel(0, |mut output| async move {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("nodes");
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(250), None, sender).unwrap();
        debouncer.watch(path, RecursiveMode::Recursive).unwrap();

        for res in receiver {
            match res {
                Ok(events) => {
                    let nodes: Vec<_> = events
                        .into_iter()
                        .map(|debounce_event| debounce_event.event)
                        .filter(|e| {
                            (e.kind.is_modify() || e.kind.is_create())
                                && e.paths.iter().any(|p| {
                                    p.extension() == Some(OsStr::new("py"))
                                        && !p
                                            .components()
                                            .any(|s| s == Component::Normal(OsStr::new(".venv")))
                                    //TODO: more reliable check here?
                                })
                        })
                        .collect();
                    if !nodes.is_empty() {
                        //println!("nodes: {nodes:?}");
                        //println!("time: {:?}", std::time::Instant::now());
                        let _ = output.send(Message::ReloadNodes).await;
                    }
                }
                Err(error) => log::error!("Error: {error:?}"),
            }
        }
    })
}

pub fn file_watch_subscription() -> Subscription<Message> {
    Subscription::run(file_watcher)
}