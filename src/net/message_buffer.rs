use crate::net::message::{Header, HEADER_SIZE, MAX_MESSAGE_SIZE, Message};

type NetBuffer = [u8; MAX_MESSAGE_SIZE];

/// A helper struct to manage reading complete [Message]`s.
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

impl MessageBuffer {
    pub fn read(&mut self, incoming_data: NetBuffer, len: usize) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut ptr = 0;

        if self.incomplete_ptr > 0 {
            // we have part of a message stored in our incomplete buffer
            // we must complete the data with the incoming data to build a valid message
            let incomplete_header =
                Header::from(&self.incomplete_buffer[ptr..HEADER_SIZE]);
            let packet_size = incomplete_header.message_size() as usize;
            let missing_data = packet_size - self.incomplete_ptr;

            // worst-case is that the inbound buffer does not contain all of the remaining data
            let size_to_copy = if len > missing_data {
                missing_data
            } else {
                len
            };
            self.incomplete_buffer[self.incomplete_ptr..size_to_copy]
                .copy_from_slice(&incoming_data[..size_to_copy]);

            if len > missing_data {
                // we were able to copy all of the remaining data!
                self.incomplete_ptr = 0;

                // build (and decode) the message and add it into the queue
                let message = Message::from(&self.incomplete_buffer[..packet_size]);
                messages.push(message);
            } else {
                // worst-case; the inbound message did not contain enough data to finish the message
                self.incomplete_ptr += size_to_copy;
            }
        }

        // there's at least one full message header in the stream which we can process
        while len - ptr >= HEADER_SIZE {
            let header = Header::from(&incoming_data[ptr..HEADER_SIZE]);
            let message_size = header.message_size();
            let recv_len = len - ptr;

            if recv_len < message_size as usize {
                // message is not fully available in this stream
                // put into buffer so that it can be completed with the next inbound
                self.incomplete_buffer.copy_from_slice(&incoming_data[ptr..]);
                self.incomplete_ptr = recv_len;
                break;
            }

            messages.push(Message::from(&incoming_data[ptr..]));

            ptr += message_size as usize;
        }

        messages
    }
}
