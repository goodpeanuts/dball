/// IPC (Inter-Process Communication) Module
///
/// provides the protocol definitions, message encapsulation,
/// and encoding/decoding functionality for inter-process communication.
pub mod client;
pub mod codec;
pub mod envelope;
pub mod protocol;

pub use codec::*;
pub use envelope::IpcEnvelope;
pub use protocol::*;
