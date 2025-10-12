//! HTTP connection handler for the HTML dialog system.
//!
//! This module provides functionality to process HTTP requests and responses
//! for the embedded web server used by the HTML dialog system. It implements
//! a simple HTTP/1.1 parser for GET and POST requests and manages the state
//! of each connection, as multiple connections share the same thread.

use std::{
    io::{BufReader, ErrorKind, prelude::*},
    net::TcpStream,
};
use log::{info, error};

/// Represents the current state of the HTTP request parsing.
///
/// The connection progresses through these states sequentially as it
/// parses an incoming HTTP request byte by byte.
#[derive(Debug, Clone, PartialEq)]
enum ConnectionStatus {
    /// Reading the HTTP verb (GET or POST)
    Verb,
    /// Reading whitespace after the verb
    Whitespace1,
    /// Reading the request location (URL path)
    Location,
    /// Reading whitespace after the location
    Whitespace2,
    /// Reading the HTTP protocol version
    Protocol,
    /// Reading the HTTP headers
    Header,
    /// Reading the request content/body
    Content,
    /// Request is completely parsed
    Complete,
    /// An error occurred during parsing
    Error,
}

/// Manages an HTTP connection over a TCP stream.
///
/// This struct handles the parsing of HTTP requests and writing of responses
/// in a non-blocking manner. It processes the request byte-by-byte and
/// maintains the state of the parsing process.
pub(crate) struct Connection {
    /// The underlying TCP stream for the connection
    stream: TcpStream,
    /// Buffered reader for the TCP stream
    reader: BufReader<TcpStream>,
    /// Current state of the HTTP request parsing
    status: ConnectionStatus,
    /// Buffer for accumulating parsed string data
    read_buffer: String,
    /// Indicates if the request is a GET (true) or POST (false)
    is_get: bool,
    /// Error message if an error occurs
    error: String,
    /// The requested URL path
    location: String,
    /// Content length from the Content-Length header
    content_length: usize,
    /// Accumulated content body for POST requests
    content: Vec<u8>,
    /// HTTP error code if an error occurs
    error_code: i32,
    /// Buffer for outgoing response data
    output: Vec<u8>,
}

impl Connection {
    /// Creates a new Connection from a TcpStream.
    ///
    /// Sets up the stream for non-blocking I/O and initializes the connection state.
    ///
    /// # Parameters
    /// * `stream` - The TCP stream for the connection
    ///
    /// # Returns
    /// * `Ok(Self)` - A new Connection instance
    /// * `Err(String)` - Error message if initialization fails
    pub fn new(stream: TcpStream) -> Result<Self, String> {
        let stream_clone = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {return Err(format!("Failed to clone tcp stream: {}", e));}
        };
        let res = Connection {
            stream,
            reader: BufReader::new(stream_clone),
            read_buffer: String::with_capacity(128),
            status: ConnectionStatus::Verb,
            is_get: false,
            location: String::new(),
            error: String::new(),
            content_length: 0,
            content: Vec::new(),
            error_code: 500, // Default error, if none more suitable
            output: Vec::new(),
        };
        match res.stream.set_nonblocking(true) {
            Ok(_) => {},
            Err(e) => {return Err(format!("Failed to set stream to non-blocking mode: {}", e));}
        };
        Ok(res)
    }

    /// Checks if the connection is in an error state.
    ///
    /// # Returns
    /// * `true` - If an error occurred during parsing
    /// * `false` - If no error has occurred
    pub fn is_error(&self) -> bool {
        self.status == ConnectionStatus::Error
    }

    /// Gets the error message if an error occurred.
    ///
    /// This error state pertained to server errors, 
    /// such as invalid headers, invalid reads, invalid
    /// writes; not to HTTP errors such as 404 Not Found.
    ///
    /// Connections that are in this state are closed
    /// and abandoned. 
    ///
    /// # Returns
    /// * The error message string
    pub fn get_error(&self) -> String {
        self.error.clone()
    }

    /// Gets the HTTP error code for the error.
    ///
    /// # Returns
    /// * The HTTP error code (e.g., 404, 500)
    /// * 0 if the connection was closed before any data was read
    pub fn get_error_code(&self) -> i32 {
        self.error_code
    }

    /// Checks if the HTTP request has been completely parsed.
    ///
    /// # Returns
    /// * `true` - If the request is fully parsed
    /// * `false` - If parsing is still in progress
    pub fn is_complete(&self) -> bool {
        self.status == ConnectionStatus::Complete
    }

    /// Gets the requested URL path.
    ///
    /// # Returns
    /// * The URL path as a string slice
    pub fn get_location(&self) -> &str {
        self.location.as_str()
    }

    /// Gets the request content (POST).
    ///
    /// # Returns
    /// * A copy of the request content as a byte vector
    pub fn get_content(&self) -> Vec<u8> {
        self.content.clone()
    }

    /// Advances the parsing of the HTTP request.
    ///
    /// This method reads and processes data from the TCP stream in a non-blocking
    /// manner, updating the connection state as it progresses through the HTTP request.
    /// It also attempts to write any pending response data.
    ///
    /// # Returns
    /// * `true` - If any data was read or written
    /// * `false` - If no data was processed
    pub fn advance(&mut self) -> bool {
        let mut any_action = false;
        loop {
            let mut char_buf = [0u8; 1];
            match self.reader.read_exact(&mut char_buf) {
                Ok(_) => {
                    let byte = char_buf[0];
                    match self.status {
                        ConnectionStatus::Verb => {
                            if byte == b' ' || byte == b'\t' {
                                self.read_buffer = self.read_buffer.to_lowercase();
                                match self.read_buffer.as_str() {
                                    "get" => {
                                        self.is_get = true;
                                    },
                                    "post" => {
                                        self.is_get = false;
                                    },
                                    _ => {
                                        self.status = ConnectionStatus::Error;
                                        self.error = format!("Invalid verb: {}", self.read_buffer);
                                        break;
                                    }
                                }
                                self.status = ConnectionStatus::Whitespace1;
                            } else if (byte as char).is_whitespace() {
                                self.status = ConnectionStatus::Error;
                                self.error = format!("Invalid request");
                                break;
                            } else {
                                self.read_buffer.push(byte as char);
                            }
                        },
                        ConnectionStatus::Whitespace1 => {
                            if byte == b'\r' || byte == b'\n' {
                                self.status = ConnectionStatus::Error;
                                self.error = format!("Invalid request");
                                break;
                            }
                            if !(byte as char).is_whitespace() {
                                self.status = ConnectionStatus::Location;
                                self.read_buffer.clear();
                                self.location.clear();
                                self.location.push(byte as char);
                            }
                        },
                        ConnectionStatus::Location => {
                            if byte == b' ' || byte == b'\t' {
                                let _ = info!("==> Location: {}", self.location);
                                self.status = ConnectionStatus::Whitespace2;
                            } else if byte == b'\r' || byte == b'\n' {
                                self.status = ConnectionStatus::Error;
                                self.error = format!("Invalid request");
                                break;
                            } else {
                                self.location.push(byte as char);
                            }
                        },
                        ConnectionStatus::Whitespace2 => {
                            if byte == b'\r' || byte == b'\n' {
                                self.status = ConnectionStatus::Error;
                                self.error = format!("Invalid request");
                                break;
                            }
                            if !(byte as char).is_whitespace() {
                                self.status = ConnectionStatus::Protocol;
                                self.read_buffer.push(byte as char);
                            }
                        },
                        ConnectionStatus::Protocol => {
                            if byte == b'\n' {
                                if self.read_buffer.to_lowercase() == "http/1.1" {
                                    self.status = ConnectionStatus::Header;
                                    self.read_buffer.clear();
                                    continue;
                                }
                                self.status = ConnectionStatus::Error;
                                self.error = format!("Invalid protocol: {}", self.read_buffer);
                                self.error_code = 505; // Unsupported protocol
                                break;
                            }
                            if !(byte as char).is_whitespace() {
                                self.read_buffer.push(byte as char);
                            }
                        },
                        ConnectionStatus::Header => {
                            if byte == b'\r' {
                                continue; // Ignore carriage return
                            }
                            if byte == b'\n' {
                                if self.read_buffer.is_empty() {
                                    // End of headers
                                    if self.content_length == 0 {
                                        self.status = ConnectionStatus::Complete;
                                        break;
                                    }
                                    self.status = ConnectionStatus::Content;
                                    break;
                                }
                                // Process header
                                let header_line = self.read_buffer.trim_end().to_lowercase();
                                let _ = info!("==> Header: {}", header_line);
                                let mut header_words = header_line.split_whitespace();
                                match header_words.next() {
                                    Some(val) => match val {
                                        "content-length:" => {
                                            match header_words.next() {
                                                Some(length) => {
                                                    match length.parse::<usize>() {
                                                        Ok(l) => {self.content_length = l;},
                                                        Err(e) => {
                                                            self.status = ConnectionStatus::Error;
                                                            self.error = format!("Invalid content length: {}", e);
                                                            break;
                                                        }
                                                    }
                                                },
                                                None => {
                                                    self.status = ConnectionStatus::Error;
                                                    self.error = "Invalid content length".into();
                                                    break;
                                                }
                                            }
                                        },
                                        _ => {}, // Ignore other headers
                                    },
                                    _ => {},
                                };
                                self.read_buffer.clear(); // Prep for next header
                                continue;
                            }
                            self.read_buffer.push(byte as char);
                        },
                        ConnectionStatus::Content => {
                            self.content.push(byte);
                            self.content_length -= 1;
                            if self.content_length == 0 {
                                self.status = ConnectionStatus::Complete;
                                break;
                            }
                        },
                        ConnectionStatus::Complete => {
                            // We should not reach here, as the connection should be closed after processing
                            assert!(false, "Unexpected state: Complete");
                            self.status = ConnectionStatus::Error;
                            self.error = "Unexpected state: Complete".to_string();
                            break;
                        },
                        ConnectionStatus::Error => {
                            // We should not reach here, as the connection should be closed after processing
                            assert!(false, "Unexpected state: Complete");
                            break;
                        },
                    };
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break; // No more data available without blocking
                }
                Err(e) => {
                    if self.status == ConnectionStatus::Verb {
                        self.error_code = 0; // Not an error, connection was closed before any data was read
                    } else {
                        self.error = format!("Read error: {}", e);
                    }
                    self.status = ConnectionStatus::Error;
                    return true;
                },
            }
        }
        if self.try_write() {
            any_action = true;
        }
        any_action
    }

    /// Resets the connection state to prepare for a new request.
    ///
    /// Clears all buffers and resets state variables to their initial values.
    pub fn reset(&mut self) {
        self.status = ConnectionStatus::Verb;
        self.read_buffer.clear();
        self.is_get = false;
        self.location.clear();
        self.error.clear();
        self.content_length = 0;
        self.content.clear();
        self.error_code = 500;
    }

    /// Writes data to the response buffer.
    ///
    /// Adds the provided data to the output buffer and attempts to write it
    /// to the TCP stream immediately.
    ///
    /// # Parameters
    /// * `data` - The byte slice to write to the response
    pub fn write(&mut self, data: &[u8]) {
        self.output.extend_from_slice(data);
        self.try_write();
    }

    /// Attempts to write pending output data to the TCP stream.
    ///
    /// This method writes data in chunks (up to 256 bytes) to avoid blocking
    /// for too long. It removes written data from the output buffer.
    ///
    /// # Returns
    /// * `true` - If any data was written
    /// * `false` - If no data was written
    fn try_write(&mut self) -> bool {
        let mut written = 0;
        loop {
            let try_size = std::cmp::min(256, self.output.len() - written);
            match self.stream.write(&self.output[written..written + try_size]) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    written += size;
                    if written >= self.output.len() {
                        break; // All data written
                    }
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No more data can be written, wait for next call
                    break;
                },
                Err(e) => {
                    let _ = error!("Failed to write data: {}", e);
                    self.status = ConnectionStatus::Error;
                    self.error = format!("Failed to write data: {}", e);
                    return true;
                },
            }
        }
        // Remove written data
        if written > 0 {
            self.output.drain(0..written);
            true
        } else {
            false
        }
    }
}

impl Drop for Connection {
    /// Ensures any remaining output data is written when the connection is dropped.
    ///
    /// This attempts to flush the output buffer before the connection is closed.
    fn drop(&mut self) {
        // Finish writing output
        if !self.output.is_empty() {
            match self.stream.write_all(&self.output) {
                Ok(_) => {
                    self.output.clear();
                },
                Err(e) => {
                    error!("Failed to write output: {}", e);
                }
            };
        }
    }
}