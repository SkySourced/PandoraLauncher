use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::message::{BridgeNotificationType, MessageToBackend, MessageToFrontend};

#[derive(Clone, Debug)]
pub struct BackendHandle {
    sender: Sender<MessageToBackend>,
}

unsafe impl Send for BackendHandle {}
unsafe impl Sync for BackendHandle {}

impl From<Sender<MessageToBackend>> for BackendHandle {
    fn from(value: Sender<MessageToBackend>) -> Self {
        Self { sender: value }
    }
}

impl BackendHandle {
    pub async fn send(&self, message: MessageToBackend) {
        let _ = self.sender.send(message).await;
    }
    
    pub fn blocking_send(&self, message: MessageToBackend) {
        let _ = self.sender.blocking_send(message);
    }
    
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

#[derive(Clone, Debug)]
pub struct FrontendHandle {
    sender: Sender<MessageToFrontend>,
}

unsafe impl Send for FrontendHandle {}
unsafe impl Sync for FrontendHandle {}

impl From<Sender<MessageToFrontend>> for FrontendHandle {
    fn from(value: Sender<MessageToFrontend>) -> Self {
        Self { sender: value }
    }
}

impl FrontendHandle {
    pub async fn send(&self, message: MessageToFrontend) {
        let _ = self.sender.send(message).await;
    }
    
    pub fn blocking_send(&self, message: MessageToFrontend) {
        let _ = self.sender.blocking_send(message);
    }
    
    pub async fn send_info(&self, info: impl Into<Arc<str>>) {
        self.send(MessageToFrontend::AddNotification {
            notification_type: BridgeNotificationType::Info,
            message: info.into()
        }).await
    }
    
    pub async fn send_success(&self, success: impl Into<Arc<str>>) {
        self.send(MessageToFrontend::AddNotification {
            notification_type: BridgeNotificationType::Success,
            message: success.into()
        }).await
    }
    
    pub async fn send_warning(&self, warning: impl Into<Arc<str>>) {
        self.send(MessageToFrontend::AddNotification {
            notification_type: BridgeNotificationType::Warning,
            message: warning.into()
        }).await
    }
    
    pub async fn send_error(&self, error: impl Into<Arc<str>>) {
        self.send(MessageToFrontend::AddNotification {
            notification_type: BridgeNotificationType::Error,
            message: error.into()
        }).await
    }
    
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}
