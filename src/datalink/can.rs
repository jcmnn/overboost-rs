use std::ffi;
use std::fmt;
use std::io;
use std::io::{Read, Write};
use std::iter;
use std::mem;
use std::result::Result;
use std::time;
use std::time::Duration;

#[cfg(unix)]
use socketcan::{CANError, CANFrame, CANSocket};

#[derive(Debug)]
pub struct Message {
    pub id: u32,
    pub data: [u8; 8],
    pub len: u8,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            id: 0,
            data: [0; 8],
            len: 0,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{:X}] {}",
            self.id,
            self.data
                .iter()
                .map(|x| format!("{:X}", x))
                .collect::<Vec<String>>()
                .join(" ")
        )?;
        Ok(())
    }
}

pub trait Can {
    /// Sends a CAN message through the interface.
    ///
    /// # Arguments
    ///
    /// * `id` - The arbitration id of the message
    /// * `message` - The message data. Must not be larger than 8 bytes
    fn write(&self, id: u32, message: &[u8]) -> std::io::Result<()>;

    fn send_msg(&self, message: &Message) -> std::io::Result<()> {
        self.write(message.id, &message.data)
    }

    /// Received a single message from the interface.
    /// If no messages are received before the timeout, returns `Error::Timeout`
    ///
    /// # Arguments
    ///
    /// * `timeout` - The time to wait for a message before returning
    fn read(&self, timeout: time::Duration) -> std::io::Result<Message>;
}

#[cfg(unix)]
impl Can for CANSocket {
    fn write(&self, id: u32, message: &[u8]) -> std::io::Result<()> {
        let frame = CANFrame::new(id, message, false, false).unwrap();
        self.write_frame_insist(&frame)
    }

    fn read(&self, timeout: Duration) -> std::io::Result<Message> {
        self.set_read_timeout(timeout)?;
        let frame = self.read_frame()?;
        let frame_data = frame.data();
        let mut data = [0_u8; 8];
        data.copy_from_slice(frame_data);
        Ok(Message {
            id: frame.id(),
            data,
            len: frame_data.len() as u8,
        })
    }
}

/*
pub struct J2534Can {
    channel: j2534::Channel,
}

impl J2534Can {
    /// Creates a new device from a J2534 channel. The channel must be a CAN channel.
    pub fn new(channel: j2534::Channel) -> Result<J2534Can> {
        Ok(J2534Can { channel })
    }

    /// Creates a CAN channel from a device with the specified baudrate
    pub fn connect(device: Rc<j2534::Device>, baudrate: u32) -> Result<J2534Can> {
        J2534Can::new(j2534::Channel::connect(device, j2534::Protocol::CAN, j2534::ConnectFlags::CAN_ID_BOTH, baudrate)?)
    }

    /// Applies a filter that wil allow all messages through
    pub fn apply_blank_filter(&self) -> Result<()> {
        let msg_mask = j2534::PassThruMsg {
            protocol_id: j2534::Protocol::CAN as u32,
            data_size: 5,
            // data[0..5] is already set to 0
            ..Default::default()
        };

        let msg_pattern = msg_mask;
        let _id = self.channel.start_msg_filter(j2534::FilterType::Pass, Some(&msg_mask), Some(&msg_pattern), None)?;
        Ok(())
    }
}

impl CanInterface for J2534Can {
    /// Sends a CAN message through the PassThru channel.
    ///
    /// # Arguments
    ///
    /// * `id` - The CAN id of the message
    /// * `message` - The message data. Must not be larger than 8 bytes
    fn send(&self, id: u32, message: &[u8]) -> Result<()> {
        if message.len() > 8 {
            return Err(Error::TooMuchData);
        }
        let data = {
            let mut d: [u8; 4128] = [0; 4128];
            {
                let mut writer: &mut [u8] = &mut d;
                writer.write_u32::<BigEndian>(id)?;
                writer.write(message)?;
            }
            d
        };
        let mut msg = [j2534::PassThruMsg::new_raw(j2534::Protocol::CAN, 0, 0, 0, message.len() as u32 + 4, 0, data)];

        // Use a timeout of 100ms
        let num_msgs = self.channel.write_msgs(&mut msg, 100)?;
        if num_msgs != 1 {
            return Err(Error::IncompleteWrite);
        }
        Ok(())
    }

    /// Received a single message from the PassThru channel.
    /// If no messages are received before the timeout, returns `Error::Timeout`
    ///
    /// # Arguments
    ///
    /// * `timeout` - The time to wait for a message before returning
    fn recv(&self, timeout: time::Duration) -> Result<Message> {
        let mut remaining = timeout;
        loop {
            let millis = (remaining.as_secs() * 1000 + remaining.subsec_millis() as u64) as u32;
            let msg = self.channel.read_msg(millis)?;
            if msg.data_size < 4 {
                continue;
            }
            let mut reader: &[u8] = &msg.data;
            let id = reader.read_u32::<BigEndian>()?;
            let mut buffer = vec![0; (msg.data_size - 4) as usize];
            let _amount = reader.read(&mut buffer)?;

            break Ok(Message {
                id,
                data: buffer,
            });
        }
    }
}*/

/*
impl CanInterface for SocketCan {
    fn send(&self, id: u32, message: &[u8]) -> Result<()> {
        if message.len() > 8 {
            return Err(Error::TooMuchData);
        }

        let mut frame = CanFrame::default();
        frame.can_dlc = message.len() as u8;
        frame.can_id = id;
        frame.data[..message.len()].clone_from_slice(message);

        let frame_ptr = &frame as *const CanFrame;

        let res = unsafe { libc::write(self.fd, frame_ptr as *const libc::c_void, mem::size_of::<CanFrame>()) };
        if res != mem::size_of::<CanFrame>() as isize {
            if res < 0 {
                return Err(Error::Io(io::Error::last_os_error()));
            }
            return Err(Error::IncompleteWrite);
        }
        Ok(())
    }

    // FIXME: implement timeout
    fn recv(&self, timeout: time::Duration) -> Result<Message> {
        let mut frame = CanFrame::default();
        let frame_ptr = &mut frame as *mut CanFrame;

        let res = unsafe { libc::recv(self.fd, frame_ptr as *mut libc::c_void, mem::size_of::<CanFrame>(), 0) };
        if res < 0 {
            return Err(Error::Io(io::Error::last_os_error()));
        }
        Ok(Message {
            id: frame.can_id,
            data: frame.data[..(frame.can_dlc as usize)].to_vec(),
        })
    }
}*/
