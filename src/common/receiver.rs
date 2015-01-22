//! The default implementation of a WebSocket Receiver.

use common::{WebSocketDataFrame, WebSocketMessage};
use common::{Local, Remote};
use result::{WebSocketResult, WebSocketError};
use ws::{Receiver, Message};
use ws::util::dataframe::read_dataframe;

/// A Receiver that wraps a Reader and provides a default implementation using
/// WebSocketDataFrames and WebSocketMessages.
pub struct WebSocketReceiver<R, L> {
	inner: R,
	buffer: Vec<WebSocketDataFrame>
}

impl<R, L> WebSocketReceiver<R, L> {
	/// Create a new WebSocketReceiver using the specified Reader.
	pub fn new(reader: R) -> WebSocketReceiver<R, L> {
		WebSocketReceiver {
			inner: reader,
			buffer: Vec::new()
		}
	}
}

impl<R: Reader> Receiver<WebSocketDataFrame> for WebSocketReceiver<R, Local> {
	type Message = WebSocketMessage;
	
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		match self.buffer.pop() {
			Some(dataframe) => Ok(dataframe),
			None => read_dataframe(&mut self.inner, false),
		}
	}
	fn recv_message(&mut self) -> WebSocketResult<WebSocketMessage> {
		let first = try!(self.recv_dataframe());
		
		let mut finished = first.finished;
		let mut data = first.data.clone();
		let mut buffer = Vec::new();
		let mut frames = Vec::new();
		
		while !finished {
			let next = try!(self.recv_dataframe());
			finished = next.finished;
			
			match next.opcode as u8 {
				// Continuation opcode
				0 => frames.push(next),
				// Control frame
				8...15 => buffer.push(next),
				// Others
				_ => return Err(WebSocketError::ProtocolError(
					"Unexpected data frame opcode".to_string()
				)),
			}
		}
		
		self.buffer.push_all(&buffer[]);

		Message::from_iter(frames.into_iter())
	}
}

impl<R: Reader> Receiver<WebSocketDataFrame> for WebSocketReceiver<R, Remote> {
	type Message = WebSocketMessage;
	
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		match self.buffer.pop() {
			Some(dataframe) => Ok(dataframe),
			None => read_dataframe(&mut self.inner, true),
		}
	}
	fn recv_message(&mut self) -> WebSocketResult<WebSocketMessage> {
		let first = try!(self.recv_dataframe());
		
		let mut finished = first.finished;
		let mut data = first.data.clone();
		let mut buffer = Vec::new();
		let mut frames = Vec::new();
		
		while !finished {
			let next = try!(self.recv_dataframe());
			finished = next.finished;
			
			match next.opcode as u8 {
				// Continuation opcode
				0 => frames.push(next),
				// Control frame
				8...15 => buffer.push(next),
				// Others
				_ => return Err(WebSocketError::ProtocolError(
					"Unexpected data frame opcode".to_string()
				)),
			}
		}
		
		self.buffer.push_all(&buffer[]);
		
		Message::from_iter(frames.into_iter())
	}
}