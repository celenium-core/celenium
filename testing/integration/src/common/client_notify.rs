use async_channel::Sender;
use celenium_notify::notifier::Notify;
use celenium_rpc_core::Notification;

#[derive(Debug)]
pub struct ChannelNotify {
    sender: Sender<Notification>,
}

impl ChannelNotify {
    pub fn new(sender: Sender<Notification>) -> Self {
        Self { sender }
    }
}

impl Notify<Notification> for ChannelNotify {
    fn notify(&self, notification: Notification) -> celenium_notify::error::Result<()> {
        self.sender.try_send(notification)?;
        Ok(())
    }
}
