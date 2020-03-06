use std::io::Cursor;
use std::result::Result;

use byteordered::ByteOrdered;
use thiserror::Error;

use crate::datalink::isotp::{Isotp, IsotpError};

pub struct Response {
    pub data: Vec<u8>,
}

// Request SIDs
pub const UDS_REQ_SESSION: u8 = 0x10;
pub const UDS_REQ_SECURITY: u8 = 0x27;
pub const UDS_REQ_READMEM: u8 = 0x23;
pub const UDS_REQ_REQUESTDOWNLOAD: u8 = 0x34;
pub const UDS_REQ_REQUESTUPLOAD: u8 = 0x35;
pub const UDS_REQ_TRANSFERDATA: u8 = 0x36;
pub const UDS_REQ_READDATABYID: u8 = 0x22;

// Negative response codes
// requestCorrectlyReceivedResponsePending
pub const UDS_NRES_RCRRP: u8 = 0x78;

#[derive(Error, Debug)]
pub enum UdsError {
    #[error(transparent)]
    Isotp(#[from] IsotpError),

    #[error("negative UDS response: {0}")]
    NegativeResponse(u8),

    #[error("empty UDS response")]
    EmptyResponse,

    #[error("invalid response id")]
    InvalidResponseId,

    #[error("invalid response data")]
    InvalidResponse,
}

pub trait UdsInterface {
    fn request(&self, request_sid: u8, data: &[u8]) -> Result<Vec<u8>, UdsError>;

    /// Sends a DiagnosticSessionControl request. Returns parameter record.
    fn request_session(&self, session_type: u8) -> Result<Vec<u8>, UdsError> {
        let mut response = self.request(UDS_REQ_SESSION, &[session_type])?;
        if response.is_empty() {
            return Err(UdsError::EmptyResponse);
        }

        if response[0] != session_type {
            return Err(UdsError::InvalidResponse);
        }

        response.remove(0);
        Ok(response)
    }

    fn request_security_seed(&self) -> Result<Vec<u8>, UdsError> {
        let mut response = self.request(UDS_REQ_SECURITY, &[1])?;

        if response.is_empty() {
            return Err(UdsError::EmptyResponse);
        }

        if response[0] != 1 {
            return Err(UdsError::InvalidResponse);
        }

        response.remove(0);
        Ok(response)
    }

    fn request_security_key(&self, key: &[u8]) -> Result<(), UdsError> {
        let mut request = Vec::with_capacity(key.len() + 1);
        request.push(2);
        request.extend_from_slice(&key);

        let _response = self.request(UDS_REQ_SECURITY, &request)?;
        Ok(())
    }

    fn request_read_memory_address(&self, address: u32, length: u16) -> Result<Vec<u8>, UdsError> {
        let mut request = [0; 6];
        {
            let mut buff = Cursor::new(&mut request as &mut [u8]);
            let mut wt = ByteOrdered::be(&mut buff);
            wt.write_u32(address).unwrap();
            wt.write_u16(length).unwrap();
        }
        self.request(UDS_REQ_READMEM, &request)
    }

    fn read_data_by_identifier(&self, id: u16) -> Result<Vec<u8>, UdsError> {
        let request = &[(id >> 8) as u8, (id & 0xFF) as u8];
        let mut res = self.request(UDS_REQ_READDATABYID, request)?;
        if res.len() < 2 {
            return Err(UdsError::InvalidResponse);
        }
        if res[0] != request[0] || res[1] != request[1] {
            // Check dataIdentifier
            return Err(UdsError::InvalidResponse);
        }
        // Remove dataIdentifier
        Ok(res.into_iter().skip(2).collect())
    }
}

impl UdsInterface for dyn Isotp {
    fn request(&self, request_sid: u8, data: &[u8]) -> Result<Vec<u8>, UdsError> {
        let mut v = Vec::new();
        v.push(request_sid);
        v.extend_from_slice(&data);

        self.write_isotp(&v)?;
        // Receive packets until we get a non-response-pending packet
        loop {
            let response = self.read_isotp()?;
            if response.is_empty() {
                return Err(UdsError::EmptyResponse);
            }

            if response[0] == 0x7F {
                // Negative code
                if response.len() > 1 {
                    if response[1] == UDS_NRES_RCRRP {
                        // Request correctly received, response pending
                        continue;
                    }
                    return Err(UdsError::NegativeResponse(response[1]));
                }
                return Err(UdsError::NegativeResponse(0));
            }

            if response[0] != request_sid + 0x40 {
                return Err(UdsError::InvalidResponseId);
            }

            return Ok(response[1..].to_vec());
        }
    }
}
