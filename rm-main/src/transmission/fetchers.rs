use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use transmission_rpc::types::{FreeSpace, SessionStats, TorrentGetField};

use crate::{
    app,
    ui::tabs::torrents::{rustmission_torrent::RustmissionTorrent, table_manager::TableManager},
};
use rm_shared::action::Action;

pub async fn stats(ctx: app::Ctx, stats: Arc<Mutex<Option<SessionStats>>>) {
    loop {
        let new_stats = ctx
            .client
            .lock()
            .await
            .session_stats()
            .await
            .unwrap()
            .arguments;
        *stats.lock().unwrap() = Some(new_stats);
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(ctx.config.connection.stats_refresh)).await;
    }
}

pub async fn free_space(ctx: app::Ctx, free_space: Arc<Mutex<Option<FreeSpace>>>) {
    let download_dir = ctx
        .client
        .lock()
        .await
        .session_get()
        .await
        .unwrap()
        .arguments
        .download_dir
        .leak();

    loop {
        let new_free_space = ctx
            .client
            .lock()
            .await
            .free_space(download_dir.to_string())
            .await
            .unwrap()
            .arguments;
        *free_space.lock().unwrap() = Some(new_free_space);
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(
            ctx.config.connection.free_space_refresh,
        ))
        .await;
    }
}

pub async fn torrents(ctx: app::Ctx, table_manager: Arc<Mutex<TableManager>>) {
    loop {
        let fields = vec![
            TorrentGetField::Id,
            TorrentGetField::Name,
            TorrentGetField::IsFinished,
            TorrentGetField::IsStalled,
            TorrentGetField::PercentDone,
            TorrentGetField::UploadRatio,
            TorrentGetField::SizeWhenDone,
            TorrentGetField::Eta,
            TorrentGetField::RateUpload,
            TorrentGetField::RateDownload,
            TorrentGetField::Status,
        ];
        let rpc_response = ctx
            .client
            .lock()
            .await
            .torrent_get(Some(fields), None)
            .await
            .unwrap();

        let new_torrents = rpc_response.arguments.torrents;

        {
            let mut table_manager_lock = table_manager.lock().unwrap();
            table_manager_lock
                .set_new_rows(new_torrents.iter().map(RustmissionTorrent::from).collect());
        }
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(ctx.config.connection.torrents_refresh)).await;
    }
}
