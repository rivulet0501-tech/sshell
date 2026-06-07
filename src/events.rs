#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEvent {
    Connected { session_id: u64, title: String },
    Output { session_id: u64, bytes: Vec<u8> },
    Errored { session_id: u64, exit_code: u8 },
    Closed { session_id: u64 },
}