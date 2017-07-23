use ConnectionError;
use error::Reason;
use frame::{self, Frame};
use proto::*;

/// Tracks END_STREAM frames sent from the local peer.
#[derive(Debug)]
pub struct StreamSendClose<T> {
    inner: T,
}

impl<T, U> StreamSendClose<T>
    where T: Stream<Item = Frame, Error = ConnectionError>,
          T: Sink<SinkItem = Frame<U>, SinkError = ConnectionError>,
          T: ControlStreams,
{
    pub fn new(inner: T) -> StreamSendClose<T> {
        StreamSendClose { inner }
    }
}

/// Proxy.
impl<T> Stream for StreamSendClose<T>
    where T: Stream<Item = Frame, Error = ConnectionError>,
          T: ControlStreams,
{
    type Item = Frame;
    type Error = ConnectionError;

    fn poll(&mut self) -> Poll<Option<Frame>, ConnectionError> {
        self.inner.poll()
    }
}

/// Tracks END_STREAM frames sent from the local peer.
impl<T, U> Sink for StreamSendClose<T>
    where T: Sink<SinkItem = Frame<U>, SinkError = ConnectionError>,
          T: ControlStreams,
{
    type SinkItem = Frame<U>;
    type SinkError = ConnectionError;

    fn start_send(&mut self, frame: Self::SinkItem) -> StartSend<Frame<U>, ConnectionError> {
        let id = frame.stream_id();
        let eos = frame.is_end_stream();
        trace!("start_send: id={:?} eos={}", id, eos);
        if !id.is_zero() {
            if eos {
                if let &Frame::Reset(ref rst) = &frame {
                    self.inner.reset_stream(id, rst.reason());
                } else {
                    debug_assert!(self.inner.is_active(id));
                    self.inner.close_send_half(id)?;
                }
            }
        }

        self.inner.start_send(frame)
    }

    fn poll_complete(&mut self) -> Poll<(), ConnectionError> {
        self.inner.poll_complete()
    }
}

/// Proxy.
impl<T, U> ReadySink for StreamSendClose<T>
    where T: Sink<SinkItem = Frame<U>, SinkError = ConnectionError>,
          T: ReadySink,
          T: ControlStreams,
{
    fn poll_ready(&mut self) -> Poll<(), ConnectionError> {
        self.inner.poll_ready()
    }
}

/// Proxy.
impl<T: ApplySettings> ApplySettings for StreamSendClose<T> {
    fn apply_local_settings(&mut self, set: &frame::SettingSet) -> Result<(), ConnectionError> {
        self.inner.apply_local_settings(set)
    }

    fn apply_remote_settings(&mut self, set: &frame::SettingSet) -> Result<(), ConnectionError> {
        self.inner.apply_remote_settings(set)
    }
}

/// Proxy.
impl<T: ControlStreams> ControlStreams for StreamSendClose<T> {
    fn local_valid_id(id: StreamId) -> bool {
        T::local_valid_id(id)
    }

    fn remote_valid_id(id: StreamId) -> bool {
        T::remote_valid_id(id)
    }

    fn local_can_open() -> bool {
        T::local_can_open()
    }

    fn local_open(&mut self, id: StreamId, sz: WindowSize) -> Result<(), ConnectionError> {
        self.inner.local_open(id, sz)
    }

    fn remote_open(&mut self, id: StreamId, sz: WindowSize) -> Result<(), ConnectionError> {
        self.inner.remote_open(id, sz)
    }

    fn local_open_recv_half(&mut self, id: StreamId, sz: WindowSize) -> Result<(), ConnectionError> {
        self.inner.local_open_recv_half(id, sz)
    }

    fn remote_open_send_half(&mut self, id: StreamId, sz: WindowSize) -> Result<(), ConnectionError> {
        self.inner.remote_open_send_half(id, sz)
    }

    fn close_send_half(&mut self, id: StreamId) -> Result<(), ConnectionError> {
        self.inner.close_send_half(id)
    }

    fn close_recv_half(&mut self, id: StreamId) -> Result<(), ConnectionError> {
        self.inner.close_recv_half(id)
    }

    fn reset_stream(&mut self, id: StreamId, cause: Reason) {
        self.inner.reset_stream(id, cause)
    }

    fn get_reset(&self, id: StreamId) -> Option<Reason> {
        self.inner.get_reset(id)
    }

    fn is_local_active(&self, id: StreamId) -> bool {
        self.inner.is_local_active(id)
    }

    fn is_remote_active(&self, id: StreamId) -> bool {
        self.inner.is_remote_active(id)
    }

    fn local_active_len(&self) -> usize {
        self.inner.local_active_len()
    }

    fn remote_active_len(&self) -> usize {
        self.inner.remote_active_len()
    }

    fn update_inital_recv_window_size(&mut self, old_sz: WindowSize, new_sz: WindowSize) {
        self.inner.update_inital_recv_window_size(old_sz, new_sz)
    }

    fn update_inital_send_window_size(&mut self, old_sz: WindowSize, new_sz: WindowSize) {
        self.inner.update_inital_send_window_size(old_sz, new_sz)
    }

    fn recv_flow_controller(&mut self, id: StreamId) -> Option<&mut FlowControlState> {
        self.inner.recv_flow_controller(id)
    }

    fn send_flow_controller(&mut self, id: StreamId) -> Option<&mut FlowControlState> {
        self.inner.send_flow_controller(id)
    }

    fn is_send_open(&mut self, id: StreamId) -> bool {
        self.inner.is_send_open(id)
    }

    fn is_recv_open(&mut self, id: StreamId) -> bool  {
        self.inner.is_recv_open(id)
    }
}

/// Proxy.
impl<T: ControlPing> ControlPing for StreamSendClose<T> {
    fn start_ping(&mut self, body: PingPayload) -> StartSend<PingPayload, ConnectionError> {
        self.inner.start_ping(body)
    }

    fn take_pong(&mut self) -> Option<PingPayload> {
        self.inner.take_pong()
    }
}
