use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Eq, PartialEq)]
pub enum RoutedKey {
    PendingPrefix,
    NextTab,
    PrevTab,
    CloseTab,
    SendToRemote(Vec<u8>),
}

#[derive(Default)]
pub struct InputRouter {
    awaiting_prefix: bool,
}

impl InputRouter {
    pub fn route(&mut self, key: KeyEvent) -> RoutedKey {
        if self.awaiting_prefix {
            self.awaiting_prefix = false;
            return match key.code {
                KeyCode::Char('n') => RoutedKey::NextTab,
                KeyCode::Char('p') => RoutedKey::PrevTab,
                KeyCode::Char('x') => RoutedKey::CloseTab,
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    RoutedKey::SendToRemote(vec![0x01])
                }
                KeyCode::Char(ch) => RoutedKey::SendToRemote(vec![ch as u8]),
                _ => RoutedKey::SendToRemote(Vec::new()),
            };
        }

        if key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.awaiting_prefix = true;
            RoutedKey::PendingPrefix
        } else {
            match key.code {
                KeyCode::Char(ch) => RoutedKey::SendToRemote(vec![ch as u8]),
                KeyCode::Enter => RoutedKey::SendToRemote(vec![b'\n']),
                _ => RoutedKey::SendToRemote(Vec::new()),
            }
        }
    }
}