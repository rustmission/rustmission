use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc::UnboundedReceiver, oneshot};
use transmission_rpc::types::{
    Id, SessionGet, Torrent, TorrentAction as RPCAction, TorrentAddArgs, TorrentSetArgs,
};

use crate::app;
use rm_shared::action::Action;
use rm_shared::action::ErrorMessage;

#[derive(Debug)]
pub enum TorrentAction {
    // Magnet/URL, Directory
    Add(String, Option<String>),
    Stop(Vec<Id>),
    Start(Vec<Id>),
    DeleteWithoutFiles(Vec<Id>),
    DeleteWithFiles(Vec<Id>),
    GetTorrentInfo(Id, Arc<Mutex<Option<Torrent>>>),
    GetSessionGet(oneshot::Sender<SessionGet>),
    SetArgs(Box<TorrentSetArgs>, Option<Vec<Id>>),
}

// TODO: make all the options use the same type of interface. Probably use a sender everywhere
pub async fn action_handler(ctx: app::Ctx, mut trans_rx: UnboundedReceiver<TorrentAction>) {
    while let Some(action) = trans_rx.recv().await {
        match action {
            TorrentAction::Add(ref url, directory) => {
                let args = TorrentAddArgs {
                    filename: Some(url.clone()),
                    download_dir: directory,
                    ..Default::default()
                };

                if let Err(e) = ctx.client.lock().await.torrent_add(args).await {
                    let error_title = "Failed to add a torrent";
                    let msg = "Failed to add torrent with URL/Path:\n\"".to_owned()
                        + url
                        + "\"\n"
                        + &e.to_string();
                    let error_message = ErrorMessage {
                        title: error_title.to_string(),
                        message: msg,
                    };
                    ctx.send_action(Action::Error(Box::new(error_message)));
                }
            }
            TorrentAction::Stop(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_action(RPCAction::Stop, ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::Start(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_action(RPCAction::Start, ids.clone())
                    .await
                    .unwrap();
            }
            TorrentAction::DeleteWithFiles(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_remove(ids, true)
                    .await
                    .unwrap();
            }
            TorrentAction::DeleteWithoutFiles(ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_remove(ids, false)
                    .await
                    .unwrap();
            }
            TorrentAction::GetTorrentInfo(id, torrent_info) => {
                let new_torrent_info = ctx
                    .client
                    .lock()
                    .await
                    .torrent_get(None, Some(vec![id]))
                    .await
                    .unwrap()
                    .arguments
                    .torrents
                    .pop()
                    .unwrap();
                *torrent_info.lock().unwrap() = Some(new_torrent_info);
            }
            TorrentAction::SetArgs(args, ids) => {
                ctx.client
                    .lock()
                    .await
                    .torrent_set(*args, ids)
                    .await
                    .unwrap();
            }
            TorrentAction::GetSessionGet(sender) => {
                let session_get = ctx
                    .client
                    .lock()
                    .await
                    .session_get()
                    .await
                    .unwrap()
                    .arguments;
                sender.send(session_get).unwrap();
            }
        }
    }
}
