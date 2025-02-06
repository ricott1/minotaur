use super::channel::AppChannel;
use super::SSHEventHandler;
use super::SSHWriterProxy;
use super::TerminalEvent;
use crate::tui::Tui;
use crate::AppResult;
use anyhow::anyhow;
use anyhow::Context;
use async_trait::async_trait;
use russh::{
    server::{self, *},
    ChannelId,
};
use russh::{Channel, Pty};
use russh_keys::PublicKey;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

pub type PlayerId = uuid::Uuid;

pub struct AppClient {
    id: PlayerId,
    username: String,
    client_sender: Sender<Tui>,
    terminal_event_sender: Sender<(PlayerId, TerminalEvent)>,
    server_shutdown: CancellationToken,
    channels: HashMap<ChannelId, AppChannel>,
}

impl AppClient {
    pub fn new(
        server_shutdown: CancellationToken,
        client_sender: Sender<Tui>,
        terminal_event_sender: Sender<(PlayerId, TerminalEvent)>,
    ) -> Self {
        AppClient {
            id: uuid::Uuid::new_v4(),
            username: "".into(),
            client_sender,
            terminal_event_sender,
            server_shutdown,
            channels: HashMap::new(),
        }
    }

    fn channel_mut(&mut self, id: ChannelId) -> AppResult<&mut AppChannel> {
        self.channels
            .get_mut(&id)
            .with_context(|| format!("unknown channel: {}", id))
    }

    pub async fn get_tui(&mut self, channel_id: ChannelId, handle: Handle) -> AppResult<()> {
        let writer = SSHWriterProxy::new(channel_id, handle);
        let client_shutdown = CancellationToken::new();

        let stdin = self.channel_mut(channel_id)?.pty_request().await?;

        SSHEventHandler::start(
            stdin,
            self.terminal_event_sender.clone(),
            self.id,
            client_shutdown.clone(),
            self.server_shutdown.clone(),
        );
        let tui = Tui::new(self.id, self.username.clone(), writer, client_shutdown)?;

        self.client_sender.send(tui).await?;

        Ok(())
    }
}

#[async_trait]
impl server::Handler for AppClient {
    type Error = anyhow::Error;

    async fn auth_none(&mut self, user: &str) -> Result<Auth, Self::Error> {
        self.username = user.to_string();
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, user: &str, _password: &str) -> Result<Auth, Self::Error> {
        self.username = user.to_string();
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        _public_key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        self.username = user.to_string();
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> AppResult<bool> {
        let app_channel = AppChannel::new();
        let created = self.channels.insert(channel.id(), app_channel).is_none();

        if created {
            println!("{} joined the game", self.username);
            Ok(true)
        } else {
            Err(anyhow!(
                "channel `{}` has been already opened",
                channel.id()
            ))
        }
    }

    async fn channel_close(&mut self, channel: ChannelId, _: &mut Session) -> AppResult<()> {
        if self.channels.remove(&channel).is_some() {
            Ok(())
        } else {
            Err(anyhow!("channel `{}` has been already closed", channel))
        }
    }

    async fn data(&mut self, id: ChannelId, data: &[u8], _: &mut Session) -> AppResult<()> {
        self.channel_mut(id)?.data(data).await?;

        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel_id: ChannelId,
        _: &str,
        _: u32,
        _: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session,
    ) -> AppResult<()> {
        self.get_tui(channel_id, session.handle()).await?;
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        id: ChannelId,
        width: u32,
        height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> AppResult<()> {
        self.channel_mut(id)?
            .window_change_request(width, height)
            .await?;

        Ok(())
    }
}
