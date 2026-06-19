use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use jadm_common::protocol::{Request, Response};
use crate::queue::manager::{QueueManager, AddDownloadParams};
use anyhow::Result;
use std::fs;

pub struct UnixRpcServer {
    queue_manager: Arc<QueueManager>,
    config: Arc<crate::config::Config>,
    path: String,
}

impl UnixRpcServer {
    pub fn new(queue_manager: Arc<QueueManager>, config: Arc<crate::config::Config>, path: String) -> Self {
        Self {
            queue_manager,
            config,
            path,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let _ = fs::remove_file(&self.path);
        let listener = UnixListener::bind(&self.path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600));
        }
        println!("Unix RPC server listening on {}", self.path);

        loop {
            let (socket, _) = listener.accept().await?;
            let queue_manager = self.queue_manager.clone();
            let config = self.config.clone();

            tokio::spawn(async move {
                let (reader, mut writer) = socket.into_split();
                let mut lines = BufReader::new(reader).lines();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    let request: Request = match serde_json::from_str(&line) {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("Unix RPC: Failed to parse JSON: {}", e);
                            let _ = writer.write_all(b"{\"error\":\"Invalid JSON\"}\n").await;
                            continue;
                        }
                    };

                    let response = match request {
                        Request::GetQueue => {
                            Response::Queue { downloads: queue_manager.get_queue() }
                        }
                        Request::AddDownload { 
                            url, 
                            folder, 
                            category, 
                            cookies, 
                            mime_type,
                            write_subs, 
                            embed_thumbnail, 
                            embed_chapters,
                            format,
                            netscape_cookies,
                            user_agent,
                            ghost_mode,
                            engine,
                            live_support,
                            live_from_start,
                            compress_video,
                            ..
                        } => {
                            let default_folder = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()) + "/Downloads";
                            let folder = folder.unwrap_or(default_folder);
                            let params = AddDownloadParams {
                                url,
                                folder,
                                category,
                                format,
                                mime_type,
                                cookies,
                                netscape_cookies,
                                user_agent,
                                ghost_mode: ghost_mode.unwrap_or(false),
                                write_subs: write_subs.unwrap_or(false),
                                embed_thumbnail: embed_thumbnail.unwrap_or(false),
                                embed_chapters: embed_chapters.unwrap_or(false),
                                engine,
                                live_support: live_support.unwrap_or(false),
                                live_from_start: live_from_start.unwrap_or(false),
                                compress_video: compress_video.unwrap_or(false),
                            };
                            match queue_manager.add_download(params).await {
                                Ok((id, folder)) => Response::Ok { 
                                    status: "Download added".to_string(),
                                    id: Some(id),
                                    folder: Some(folder),
                                },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::GetFormats {
                            url,
                            cookies,
                            netscape_cookies,
                            user_agent,
                            mode,
                        } => {
                            match crate::ytdlp::runner::get_formats(&url, cookies, netscape_cookies, user_agent, mode).await {
                                Ok(formats) => Response::Formats {
                                    status: "ok".to_string(),
                                    formats,
                                },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::PauseDownload { id } => {
                            match queue_manager.pause_download(id).await {
                                Ok(_) => Response::Ok { status: "Paused".to_string(), id: None, folder: None },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::ResumeDownload { id } => {
                            match queue_manager.resume_download(id).await {
                                Ok(_) => Response::Ok { status: "Resumed".to_string(), id: None, folder: None },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::StopDownload { id } => {
                            match queue_manager.stop_download(id).await {
                                Ok(_) => Response::Ok { status: "Stopped".to_string(), id: None, folder: None },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::DeleteDownload { id, delete_file } => {
                            match queue_manager.delete_download(id, delete_file).await {
                                Ok(_) => Response::Ok { status: "Deleted".to_string(), id: None, folder: None },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::SiphonChunk {
                            daemon_id,
                            chunk_index,
                            is_last,
                            filename,
                            total_size,
                            data,
                        } => {
                            match uuid::Uuid::parse_str(&daemon_id) {
                                Ok(id) => {
                                    match queue_manager.handle_siphon_chunk(id, chunk_index, is_last, filename, total_size, data).await {
                                        Ok(_) => Response::Ok { status: "Chunk written".to_string(), id: None, folder: None },
                                        Err(e) => Response::Error { error: e.to_string() },
                                    }
                                }
                                Err(e) => Response::Error { error: format!("Invalid daemon UUID: {}", e) },
                            }
                        }
                        Request::StopSiphon { daemon_id } => {
                            match uuid::Uuid::parse_str(&daemon_id) {
                                Ok(id) => {
                                    match queue_manager.stop_download(id).await {
                                        Ok(_) => Response::Ok { status: "Siphon stopped".to_string(), id: None, folder: None },
                                        Err(e) => Response::Error { error: e.to_string() },
                                    }
                                }
                                Err(e) => Response::Error { error: format!("Invalid daemon UUID: {}", e) },
                            }
                        }
                        Request::CdmStart { .. } => {
                            match queue_manager.start_cdm_extractor().await {
                                Ok(_) => Response::Ok { status: "CDM extractor started successfully".to_string(), id: None, folder: None },
                                Err(e) => Response::Error { error: e.to_string() },
                            }
                        }
                        Request::Float => {
                            let cmd = config.hooks.on_float_toggle.clone();
                            if !cmd.is_empty() {
                                tokio::spawn(async move {
                                    let mut child = tokio::process::Command::new("sh");
                                    child.arg("-c").arg(&cmd);
                                    if let Err(e) = child.status().await {
                                        eprintln!("Failed to execute float toggle hook: {}", e);
                                    }
                                });
                            }
                            Response::Ok { status: "Float triggered".to_string(), id: None, folder: None }
                        }
                        _ => Response::Error { error: "Command not implemented over Unix socket yet".to_string() },
                    };

                    let mut response_json = serde_json::to_vec(&response).unwrap();
                    response_json.push(b'\n');
                    if writer.write_all(&response_json).await.is_err() {
                        return;
                    }
                }
            });
        }
    }
}
