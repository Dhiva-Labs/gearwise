//! HID++ framing, request/response matching and notification routing.
//!
//! Every HID++ 2.0 request here is sent as a *long* report (0x11, 20 bytes):
//! all HID++ 2.0 devices accept long requests for any function, and it keeps
//! Windows simple, where short and long reports live on separate HID
//! collections. Short (0x10) reports are still *read* — HID++ 1.0 errors from
//! receivers arrive that way — from the optional short-collection handle.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use periph_core::transport::SharedTransport;
use periph_core::{Error, Result};

pub const REPORT_SHORT: u8 = 0x10;
pub const REPORT_LONG: u8 = 0x11;
pub const SHORT_LEN: usize = 7;
pub const LONG_LEN: usize = 20;

/// Device index for wired / Bluetooth devices talking HID++ directly.
pub const DIRECT_INDEX: u8 = 0xff;

/// HID++ 1.0 error sub-id (also used by receivers for unreachable devices).
const ERR_HIDPP10: u8 = 0x8f;
/// HID++ 2.0 error "feature index".
const ERR_HIDPP20: u8 = 0xff;

const RESPONSE_TIMEOUT: Duration = Duration::from_millis(800);

/// HID++ 1.0 error codes worth recognising.
pub mod err10 {
    pub const INVALID_SUBID: u8 = 0x01;
    pub const UNKNOWN_DEVICE: u8 = 0x08;
    pub const RESOURCE_ERROR: u8 = 0x09;
}

/// HID++ 2.0 error codes.
pub mod err20 {
    pub const INVALID_ARGUMENT: u8 = 0x02;
    pub const OUT_OF_RANGE: u8 = 0x03;
    pub const BUSY: u8 = 0x08;
    pub const UNSUPPORTED: u8 = 0x09;
}

/// A parsed incoming HID++ message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub report_id: u8,
    pub device_index: u8,
    /// Feature index (2.0) or sub-id (1.0).
    pub feature_index: u8,
    /// Function in the high nibble, software id in the low nibble (2.0).
    pub address: u8,
    pub params: Vec<u8>,
}

impl Message {
    pub fn parse(buf: &[u8]) -> Option<Self> {
        let len = match *buf.first()? {
            REPORT_SHORT => SHORT_LEN,
            REPORT_LONG => LONG_LEN,
            _ => return None,
        };
        if buf.len() < 4 {
            return None;
        }
        let end = len.min(buf.len());
        Some(Message {
            report_id: buf[0],
            device_index: buf[1],
            feature_index: buf[2],
            address: buf[3],
            params: buf[4..end].to_vec(),
        })
    }

    /// Notification function number (high nibble of the address).
    pub fn function(&self) -> u8 {
        self.address >> 4
    }

    pub fn sw_id(&self) -> u8 {
        self.address & 0x0f
    }
}

/// Protocol-level failure reported by the device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidppError {
    /// HID++ 1.0 error (from a receiver, or a 1.0 device).
    V1(u8),
    /// HID++ 2.0 feature error.
    V2(u8),
}

impl HidppError {
    pub fn into_error(self) -> Error {
        match self {
            HidppError::V1(err10::UNKNOWN_DEVICE | err10::RESOURCE_ERROR) => Error::Disconnected,
            HidppError::V1(c) => Error::protocol(format!("HID++ 1.0 error 0x{c:02x}")),
            HidppError::V2(err20::UNSUPPORTED) => Error::Unsupported("rejected by device firmware"),
            HidppError::V2(c @ (err20::INVALID_ARGUMENT | err20::OUT_OF_RANGE)) => {
                Error::invalid(format!("device rejected value (HID++ error 0x{c:02x})"))
            }
            HidppError::V2(c) => Error::protocol(format!("HID++ 2.0 error 0x{c:02x}")),
        }
    }
}

/// Outcome of a request: the reply parameters, or a device-side error.
pub type Reply = std::result::Result<Vec<u8>, HidppError>;

struct Inner {
    long: SharedTransport,
    /// Separate handle for short reports (Windows short collection).
    short: Option<SharedTransport>,
    /// Unsolicited messages, per device index.
    pending: HashMap<u8, VecDeque<Message>>,
    next_sw_id: u8,
}

/// One HID++ interface, shared by every device index behind it.
#[derive(Clone)]
pub struct Channel {
    inner: Arc<Mutex<Inner>>,
    timeout: Duration,
}

impl Channel {
    pub fn new(long: SharedTransport, short: Option<SharedTransport>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                long,
                short,
                pending: HashMap::new(),
                next_sw_id: 0x0a,
            })),
            timeout: RESPONSE_TIMEOUT,
        }
    }

    #[cfg(test)]
    pub fn with_timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    /// Call `function` of the feature at `feature_index` on device `index`.
    ///
    /// The outer `Result` is transport failure; the inner is a protocol error
    /// reply from the device.
    pub fn request(
        &self,
        index: u8,
        feature_index: u8,
        function: u8,
        params: &[u8],
    ) -> Result<Reply> {
        let mut inner = self.inner.lock().unwrap();
        // Rotate software ids 0xa..=0xf so our replies never match the
        // kernel driver's (sw id 1) or notifications (sw id 0).
        let sw_id = inner.next_sw_id;
        inner.next_sw_id = if sw_id >= 0x0f { 0x0a } else { sw_id + 1 };

        let address = (function << 4) | sw_id;
        let mut out = [0u8; LONG_LEN];
        out[0] = REPORT_LONG;
        out[1] = index;
        out[2] = feature_index;
        out[3] = address;
        let n = params.len().min(LONG_LEN - 4);
        out[4..4 + n].copy_from_slice(&params[..n]);
        inner.long.lock().unwrap().write(&out)?;

        let deadline = Instant::now() + self.timeout;
        loop {
            let Some(msg) = inner.read_one(deadline)? else {
                return Err(Error::Timeout);
            };
            if msg.device_index != index {
                if is_notification(&msg) {
                    inner.queue(msg);
                }
                continue;
            }
            if msg.feature_index == feature_index && msg.address == address {
                return Ok(Ok(msg.params));
            }
            // Errors echo our feature index and address in the first bytes.
            let is_err = msg.feature_index == ERR_HIDPP10 || msg.feature_index == ERR_HIDPP20;
            if is_err && msg.address == feature_index && msg.params.first() == Some(&address) {
                let code = msg.params.get(1).copied().unwrap_or(0);
                return Ok(Err(if msg.feature_index == ERR_HIDPP10 {
                    HidppError::V1(code)
                } else {
                    HidppError::V2(code)
                }));
            }
            // Replies to someone else (kernel driver, other sw id) are dropped;
            // notifications are kept for poll_events.
            if is_notification(&msg) {
                inner.queue(msg);
            }
        }
    }

    /// Like [`request`](Self::request) but folds protocol errors into
    /// [`Error`] and retries BUSY a few times.
    pub fn call(
        &self,
        index: u8,
        feature_index: u8,
        function: u8,
        params: &[u8],
    ) -> Result<Vec<u8>> {
        for attempt in 0..4 {
            match self.request(index, feature_index, function, params)? {
                Ok(p) => return Ok(p),
                Err(HidppError::V2(err20::BUSY)) if attempt < 3 => {
                    std::thread::sleep(Duration::from_millis(20 * (attempt + 1) as u64));
                }
                Err(e) => return Err(e.into_error()),
            }
        }
        unreachable!("loop returns on the last attempt")
    }

    /// Pull any unsolicited messages for `index`, reading what's waiting on
    /// the wire without blocking.
    pub fn drain(&self, index: u8) -> Result<Vec<Message>> {
        let mut inner = self.inner.lock().unwrap();
        while let Some(msg) = inner.read_one(Instant::now())? {
            if is_notification(&msg) {
                inner.queue(msg);
            }
        }
        Ok(inner
            .pending
            .remove(&index)
            .map(Vec::from)
            .unwrap_or_default())
    }
}

/// HID++ 2.0 notifications carry software id 0; receivers announce device
/// (dis)connection with HID++ 1.0 sub-ids 0x40/0x41.
fn is_notification(msg: &Message) -> bool {
    msg.sw_id() == 0 || matches!(msg.feature_index, 0x40 | 0x41)
}

impl Inner {
    fn queue(&mut self, msg: Message) {
        let q = self.pending.entry(msg.device_index).or_default();
        // Bound the queue: a device nobody polls must not grow memory.
        if q.len() >= 64 {
            q.pop_front();
        }
        q.push_back(msg);
    }

    /// Read one HID++ message from either handle, or `None` once the deadline
    /// passes. Non-HID++ reports (mouse movement on shared nodes) are skipped.
    fn read_one(&mut self, deadline: Instant) -> Result<Option<Message>> {
        let mut buf = [0u8; 64];
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            // With two handles, poll each briefly in turn.
            let slice = if self.short.is_some() {
                remaining.min(Duration::from_millis(5))
            } else {
                remaining
            };
            let ms = slice.as_millis().min(i32::MAX as u128) as i32;

            let n = self.long.lock().unwrap().read_timeout(&mut buf, ms)?;
            if n > 0 {
                if let Some(m) = Message::parse(&buf[..n]) {
                    return Ok(Some(m));
                }
                continue;
            }
            if let Some(short) = &self.short {
                let n = short.lock().unwrap().read_timeout(&mut buf, 0)?;
                if n > 0 {
                    if let Some(m) = Message::parse(&buf[..n]) {
                        return Ok(Some(m));
                    }
                    continue;
                }
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use periph_core::transport::{MockOp, MockReply, MockTransport, share};

    fn long(idx: u8, feat: u8, addr: u8, params: &[u8]) -> Vec<u8> {
        let mut v = vec![REPORT_LONG, idx, feat, addr];
        v.extend_from_slice(params);
        v.resize(LONG_LEN, 0);
        v
    }

    #[test]
    fn request_matches_reply_and_skips_noise() {
        let t = MockTransport::new(|op| match op {
            MockOp::Write(d) => MockReply {
                input_reports: vec![
                    vec![0x02, 0, 5, 5],                         // mouse movement on the same node
                    long(0x02, d[2], 0x00, &[9]),                // another device's notification
                    long(d[1], d[2], (d[3] & 0xf0) | 0x1, &[7]), // kernel driver's reply (sw id 1)
                    long(d[1], d[2], d[3], &[1, 2, 3]),
                ],
                feature: None,
            },
            MockOp::SetFeature(_) => MockReply::none(),
        });
        let ch = Channel::new(share(Box::new(t)), None);
        let reply = ch.request(0xff, 0x05, 0x1, &[0xaa]).unwrap().unwrap();
        assert_eq!(&reply[..3], &[1, 2, 3]);
        // The other device's notification was queued for it.
        let pending = ch.drain(0x02).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].params[0], 9);
    }

    #[test]
    fn hidpp20_error_is_decoded() {
        let t = MockTransport::new(|op| match op {
            MockOp::Write(d) => {
                MockReply::input(long(d[1], 0xff, d[2], &[d[3], err20::INVALID_ARGUMENT]))
            }
            MockOp::SetFeature(_) => MockReply::none(),
        });
        let ch = Channel::new(share(Box::new(t)), None);
        assert_eq!(
            ch.request(0xff, 0x07, 0x3, &[]).unwrap(),
            Err(HidppError::V2(err20::INVALID_ARGUMENT))
        );
        assert!(matches!(
            ch.call(0xff, 0x07, 0x3, &[]),
            Err(Error::InvalidValue(_))
        ));
    }

    #[test]
    fn receiver_error_on_short_handle_means_disconnected() {
        let long_t = MockTransport::new(|_| MockReply::none());
        let log = long_t.log();
        let mut short_t = MockTransport::new(|_| MockReply::none());
        // Pre-load the short handle with the receiver's answer to our first
        // request (sw id 0xa, function 1, feature 0).
        short_t.push_input(vec![
            REPORT_SHORT,
            0x03,
            ERR_HIDPP10,
            0x00,
            0x1a,
            err10::UNKNOWN_DEVICE,
            0,
        ]);
        let ch = Channel::new(share(Box::new(long_t)), Some(share(Box::new(short_t))))
            .with_timeout(Duration::from_millis(50));
        let r = ch.request(0x03, 0x00, 0x1, &[0, 0, 0x5a]).unwrap();
        assert_eq!(r, Err(HidppError::V1(err10::UNKNOWN_DEVICE)));
        assert!(matches!(r.unwrap_err().into_error(), Error::Disconnected));
        // Request went out as a long report.
        let rec = log.lock().unwrap();
        let periph_core::transport::MockRecord::Write(w) = &rec[0] else {
            panic!()
        };
        assert_eq!(&w[..7], &[REPORT_LONG, 0x03, 0x00, 0x1a, 0, 0, 0x5a]);
        assert_eq!(w.len(), LONG_LEN);
    }

    #[test]
    fn timeout_when_device_is_silent() {
        let t = MockTransport::new(|_| MockReply::none());
        let ch = Channel::new(share(Box::new(t)), None).with_timeout(Duration::from_millis(10));
        assert!(matches!(ch.request(0xff, 1, 0, &[]), Err(Error::Timeout)));
    }

    #[test]
    fn busy_is_retried() {
        let mut calls = 0;
        let t = MockTransport::new(move |op| match op {
            MockOp::Write(d) => {
                calls += 1;
                if calls == 1 {
                    MockReply::input(long(d[1], 0xff, d[2], &[d[3], err20::BUSY]))
                } else {
                    MockReply::input(long(d[1], d[2], d[3], &[0x42]))
                }
            }
            MockOp::SetFeature(_) => MockReply::none(),
        });
        let ch = Channel::new(share(Box::new(t)), None);
        assert_eq!(ch.call(0xff, 4, 2, &[]).unwrap()[0], 0x42);
    }

    #[test]
    fn sw_ids_rotate_within_a_to_f() {
        let t = MockTransport::new(|op| match op {
            MockOp::Write(d) => MockReply::input(long(d[1], d[2], d[3], &[])),
            MockOp::SetFeature(_) => MockReply::none(),
        });
        let log = t.log();
        let ch = Channel::new(share(Box::new(t)), None);
        for _ in 0..8 {
            ch.request(0xff, 1, 0, &[]).unwrap().unwrap();
        }
        let ids: Vec<u8> = log
            .lock()
            .unwrap()
            .iter()
            .map(|r| match r {
                periph_core::transport::MockRecord::Write(w) => w[3] & 0x0f,
                _ => 0,
            })
            .collect();
        assert_eq!(ids, vec![0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0xa, 0xb]);
    }
}
