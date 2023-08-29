use crate::net::message::{Header, Message, HEADER_SIZE, MAX_MESSAGE_SIZE};

type NetBuffer = [u8; MAX_MESSAGE_SIZE];

/// A helper struct to manage reading complete [Message]s.
///
/// This struct acts as a buffer that accumulates incoming data until
/// a full [Message] is formed. If the incoming data stream does not
/// contain a complete [Message], this struct will retain the partial
/// data until subsequent reads supply the remaining portion to
/// complete the [Message].
pub struct MessageBuffer {
    incomplete_ptr: usize,
    incomplete_buffer: NetBuffer,
}

impl Default for MessageBuffer {
    /// Create a new empty `MessageBuffer`.
    fn default() -> Self {
        Self {
            incomplete_ptr: 0,
            incomplete_buffer: [0; MAX_MESSAGE_SIZE],
        }
    }
}

impl MessageBuffer {
    /// Handle the case where we have a partial message left in our buffer from a previous read.
    fn handle_incomplete_buffer(&mut self, incoming_data: &NetBuffer, len: usize) {
        let incomplete_header = Header::from(&self.incomplete_buffer[..HEADER_SIZE]);
        let packet_size = incomplete_header.message_size() as usize;
        let missing_data = packet_size - self.incomplete_ptr;
        let size_to_copy = std::cmp::min(len, missing_data);

        self.incomplete_buffer[self.incomplete_ptr..self.incomplete_ptr + size_to_copy]
            .copy_from_slice(&incoming_data[..size_to_copy]);

        self.incomplete_ptr += size_to_copy;
    }

    /// Read incoming data and return a vector of complete [Message]s.
    pub fn read(&mut self, incoming_data: NetBuffer, len: usize) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut ptr = 0;

        // Check if we have a partial message left from a previous read
        if self.incomplete_ptr > 0 {
            self.handle_incomplete_buffer(&incoming_data, len);

            if self.incomplete_ptr < HEADER_SIZE {
                return messages;
            }

            let incomplete_header = Header::from(&self.incomplete_buffer[..HEADER_SIZE]);
            let packet_size = incomplete_header.message_size() as usize;

            if self.incomplete_ptr == packet_size {
                let message = Message::from(&self.incomplete_buffer[..packet_size]);
                messages.push(message);
                self.incomplete_ptr = 0;
            }

            ptr = self.incomplete_ptr;
        }

        // Process the remaining incoming data
        while len - ptr >= HEADER_SIZE {
            let header = Header::from(&incoming_data[ptr..ptr + HEADER_SIZE]);
            let message_size = header.message_size() as usize;
            let recv_len = len - ptr;

            if recv_len < message_size {
                self.incomplete_buffer
                    .copy_from_slice(&incoming_data[ptr..]);
                self.incomplete_ptr = recv_len;
                break;
            }

            messages.push(Message::from(&incoming_data[ptr..ptr + message_size]));
            ptr += message_size;
        }

        messages
    }
}

#[cfg(tests)]
mod tests {
    use crate::net::message::{Header, Message, MessageId, HEADER_SIZE, MAX_MESSAGE_SIZE};
    use crate::net::MessageBuffer;
    use bytes::{BufMut, Bytes, BytesMut};

    #[test]
    fn read_complete_message_in_one_go() {
        let mut buffer = MessageBuffer::default();

        let mut data = BytesMut::new();
        data.put_u8(0);
        data.put_u8(1);
        data.put_u8(2);

        let message = Message::new(
            Header::new(MessageId::from(0x5000), data.len() as u16),
            data.clone().freeze(),
        );

        let message_bytes: Bytes = message.into();

        let mut net_buffer = [0; MAX_MESSAGE_SIZE];
        net_buffer[..message_bytes.len()].copy_from_slice(message_bytes.as_ref());

        let messages = buffer.read(net_buffer, 14);
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            Message::new(
                Header::new(MessageId::from(0x5000), data.len() as u16),
                data.clone().freeze()
            )
        );
    }

    #[test]
    fn read_split_message_across_reads() {
        let mut buffer = MessageBuffer::default();

        let mut data = BytesMut::new();
        data.put_u8(0);
        data.put_u8(1);
        data.put_u8(2);

        let message = Message::new(
            Header::new(MessageId::from(0x5000), data.len() as u16),
            data.clone().freeze(),
        );

        let message_bytes: Bytes = message.into();
        let first_bytes = &message_bytes.as_ref()[..HEADER_SIZE];
        let second_bytes = &message_bytes.as_ref()[HEADER_SIZE..];

        let mut first_net_buffer = [0; MAX_MESSAGE_SIZE];
        first_net_buffer[..HEADER_SIZE].copy_from_slice(first_bytes);

        let mut second_net_buffer = [0; MAX_MESSAGE_SIZE];
        second_net_buffer[..data.len()].copy_from_slice(second_bytes);

        let messages = buffer.read(first_net_buffer, HEADER_SIZE);
        assert_eq!(messages.len(), 0);

        let messages = buffer.read(second_net_buffer, 3);
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            Message::new(
                Header::new(MessageId::from(0x5000), data.len() as u16),
                data.clone().freeze()
            )
        );
    }
}
