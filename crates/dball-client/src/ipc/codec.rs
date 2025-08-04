use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Deserialize;
use std::io::{Cursor, Read as _, Write as _};

use super::envelope::IpcEnvelope;

pub struct IpcCodec;

impl IpcCodec {
    const COMPRESSION_THRESHOLD: usize = 1024;

    pub fn encode(envelope: &IpcEnvelope) -> Result<Vec<u8>, CodecError> {
        let json_data = serde_json::to_vec(envelope)
            .map_err(|e| CodecError::SerializationError(e.to_string()))?;

        // check if compression is needed
        let (compressed, data) = if json_data.len() > Self::COMPRESSION_THRESHOLD {
            let compressed_data = Self::compress(&json_data)?;
            (1u8, compressed_data)
        } else {
            (0u8, json_data)
        };

        let mut frame = Vec::new();

        // write data length (4 bytes, big-endian)
        let data_len = data.len() + 1; // +1 for compression flag
        frame
            .write_all(&(data_len as u32).to_be_bytes())
            .map_err(|e| CodecError::IoError(e.to_string()))?;

        // write compression flag (1 byte)
        frame
            .write_all(&[compressed])
            .map_err(|e| CodecError::IoError(e.to_string()))?;

        // write data
        frame
            .write_all(&data)
            .map_err(|e| CodecError::IoError(e.to_string()))?;

        Ok(frame)
    }

    /// decode byte stream to message
    ///
    /// return decoded message and consumed bytes
    pub fn decode(buffer: &[u8]) -> Result<Option<(IpcEnvelope, usize)>, CodecError> {
        if buffer.len() < 4 {
            // need more data to read length
            return Ok(None);
        }

        // read data length (4 bytes, big-endian)
        let data_len = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

        if buffer.len() < 4 + data_len {
            // need more data to read complete message
            return Ok(None);
        }

        // read compression flag (1 byte)
        let compressed = buffer[4];

        // read data part
        let data = &buffer[5..4 + data_len];

        // decompress if needed
        let json_data = if compressed == 1 {
            Self::decompress(data)?
        } else {
            data.to_vec()
        };

        // deserialize JSON to IpcEnvelope
        let envelope = serde_json::from_slice(&json_data)
            .map_err(|e| CodecError::DeserializationError(e.to_string()))?;

        Ok(Some((envelope, 4 + data_len)))
    }

    /// compress data
    fn compress(data: &[u8]) -> Result<Vec<u8>, CodecError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(|e| CodecError::CompressionError(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| CodecError::CompressionError(e.to_string()))
    }

    /// decompress data
    fn decompress(data: &[u8]) -> Result<Vec<u8>, CodecError> {
        let mut decoder = GzDecoder::new(Cursor::new(data));
        let mut result = Vec::new();
        decoder
            .read_to_end(&mut result)
            .map_err(|e| CodecError::CompressionError(e.to_string()))?;
        Ok(result)
    }
}

/// Error types for IPC codec operations
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid frame format")]
    InvalidFrame,
}

/// Frame buffer for handling incomplete messages
pub struct FrameBuffer {
    buffer: Vec<u8>,
}

impl FrameBuffer {
    /// Create a new  `FrameBuffer`
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Add data to the buffer
    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Try to decode a complete message from the buffer
    ///
    /// If a complete message is found, it is returned and the buffer is updated to remove the consumed data.
    /// If no complete message is found, None is returned.
    pub fn try_decode<T: for<'de> Deserialize<'de>>(
        &mut self,
    ) -> Result<Option<IpcEnvelope>, CodecError> {
        match IpcCodec::decode(&self.buffer)? {
            Some((envelope, consumed)) => {
                // update buffer to remove consumed data
                self.buffer.drain(0..consumed);
                Ok(Some(envelope))
            }
            None => Ok(None),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{envelope::IpcKind, protocol::HelloMessage};

    #[test]
    fn test_encode_decode_small_message() {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_owned()),
            server_name: None,
            supported_features: vec!["basic".to_owned()],
        };

        let envelope = IpcEnvelope::new(
            IpcKind::Hello,
            serde_json::to_value(&hello_msg).expect("Failed to serialize"),
        );

        // 编码
        let encoded = IpcCodec::encode(&envelope).expect("Failed to encode");

        // 解码
        let (decoded, consumed) = IpcCodec::decode(&encoded)
            .expect("Failed to decode")
            .expect("No message decoded");

        let received_hello_message = serde_json::from_value::<HelloMessage>(decoded.msg)
            .expect("Failed to deserialize HelloMessage");

        assert_eq!(consumed, encoded.len());
        assert_eq!(envelope.uuid, decoded.uuid);
        assert_eq!(hello_msg.client_info, received_hello_message.client_info);
    }

    #[test]
    fn test_encode_decode_large_message() {
        // 创建一个大消息来触发压缩
        let large_features: Vec<String> = (0..1000).map(|i| format!("feature_{i}")).collect();

        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_owned()),
            server_name: None,
            supported_features: large_features.clone(),
        };

        let envelope = IpcEnvelope::new(
            IpcKind::Hello,
            serde_json::to_value(&hello_msg).expect("Failed to serialize"),
        );

        // 编码
        let encoded = IpcCodec::encode(&envelope).expect("Failed to encode");

        // 解码
        let (decoded, consumed) = IpcCodec::decode(&encoded)
            .expect("Failed to decode")
            .expect("No message decoded");

        let received_hello_message = serde_json::from_value::<HelloMessage>(decoded.msg)
            .expect("Failed to deserialize HelloMessage");

        assert_eq!(consumed, encoded.len());
        assert_eq!(envelope.uuid, decoded.uuid);
        assert_eq!(
            large_features.len(),
            received_hello_message.supported_features.len()
        );
    }

    #[test]
    fn test_frame_buffer() {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_owned()),
            server_name: None,
            supported_features: vec!["basic".to_owned()],
        };

        let envelope = IpcEnvelope::new(
            IpcKind::Hello,
            serde_json::to_value(&hello_msg).expect("Failed to serialize"),
        );
        let encoded = IpcCodec::encode(&envelope).expect("Failed to encode");

        let mut buffer = FrameBuffer::new();

        // 分批添加数据
        let mid = encoded.len() / 2;
        buffer.push(&encoded[0..mid]);

        // 尝试解码，应该返回None（数据不完整）
        let result = buffer.try_decode::<HelloMessage>().expect("Decode failed");
        assert!(result.is_none());

        // 添加剩余数据
        buffer.push(&encoded[mid..]);

        // 现在应该能解码成功
        let decoded = buffer
            .try_decode::<HelloMessage>()
            .expect("Decode failed")
            .expect("No message decoded");

        assert_eq!(envelope.uuid, decoded.uuid);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_partial_frame() {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_owned()),
            server_name: None,
            supported_features: vec!["basic".to_owned()],
        };

        let envelope = IpcEnvelope::new(
            IpcKind::Hello,
            serde_json::to_value(&hello_msg).expect("Failed to serialize"),
        );
        let encoded = IpcCodec::encode(&envelope).expect("Failed to encode");

        // 测试部分数据解码
        let partial = &encoded[0..2]; // 只有部分长度字段
        let result = IpcCodec::decode(partial).expect("Decode failed");
        assert!(result.is_none());

        // 测试只有长度字段的情况
        let partial = &encoded[0..4]; // 只有长度字段
        let result = IpcCodec::decode(partial).expect("Decode failed");
        assert!(result.is_none());
    }
}
