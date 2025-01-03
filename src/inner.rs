use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use kaspa_notify::listener::ListenerId;
use kaspa_rpc_core::Notification;
use kaspa_wrpc_client::KaspaRpcClient;
use polodb_core::Collection;
use workflow_core::channel::{Channel, DuplexChannel};
use crate::message::Message;

pub(crate) struct Inner {
    // task control duplex channel - a pair of channels where sender
    // is used to signal an async task termination request and receiver
    // is used to signal task termination completion.
    pub(crate) task_ctl: DuplexChannel<()>,
    // Kaspa wRPC client instance
    pub(crate) client: Arc<KaspaRpcClient>,
    // our own view on the connection state
    pub(crate) is_connected: AtomicBool,
    // channel supplied to the notification subsystem
    // to receive the node notifications we subscribe to
    pub(crate) notification_channel: Channel<Notification>,
    // listener id used to manage notification scopes
    // we can have multiple IDs for different scopes
    // paired with multiple notification channels
    pub(crate) listener_id: Mutex<Option<ListenerId>>,
    pub(crate) stored_messages: Collection<Message>,
}