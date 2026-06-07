use std::num::NonZeroU64;

use crate::error::AppError;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionId(NonZeroU64);

impl SessionId {
    pub fn new(value: u64) -> Self {
        Self(NonZeroU64::new(value).expect("session id must be non-zero"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMeta {
    pub id: SessionId,
    pub title: String,
    pub last_error: Option<AppError>,
}

pub struct SessionManager {
    next_id: u64,
    order: Vec<SessionMeta>,
    active_index: Option<usize>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            order: Vec::new(),
            active_index: None,
        }
    }

    pub fn insert(&mut self, title: String) -> SessionId {
        let id = SessionId::new(self.next_id);
        self.next_id += 1;
        self.order.push(SessionMeta {
            id,
            title,
            last_error: None,
        });
        self.active_index = Some(self.order.len() - 1);
        id
    }

    pub fn activate(&mut self, id: SessionId) -> Result<(), &'static str> {
        let index = self
            .order
            .iter()
            .position(|session| session.id == id)
            .ok_or("unknown session")?;
        self.active_index = Some(index);
        Ok(())
    }

    pub fn close(&mut self, id: SessionId) -> Result<(), &'static str> {
        let index = self
            .order
            .iter()
            .position(|session| session.id == id)
            .ok_or("unknown session")?;
        self.order.remove(index);

        if self.order.is_empty() {
            self.active_index = None;
        } else if index >= self.order.len() {
            self.active_index = Some(self.order.len() - 1);
        } else {
            self.active_index = Some(index);
        }

        Ok(())
    }

    pub fn active_id(&self) -> Option<SessionId> {
        self.active_index.map(|index| self.order[index].id)
    }

    pub fn ordered_ids(&self) -> Vec<SessionId> {
        self.order.iter().map(|item| item.id).collect()
    }
}