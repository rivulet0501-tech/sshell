use crate::cli::CliArgs;
use crate::error::ExitAggregator;
use crate::session_manager::SessionManager;

pub struct AppState {
    pub session_manager: SessionManager,
    pub exit_aggregator: ExitAggregator,
}

#[derive(Clone, Copy)]
pub enum LocalCommand {
    NextTab,
    PrevTab,
    CloseActiveTab,
}

pub fn build_initial_state(args: CliArgs) -> AppState {
    let mut session_manager = SessionManager::new();
    for connection in args.connections {
        session_manager.insert(connection.display_name());
    }

    AppState {
        session_manager,
        exit_aggregator: ExitAggregator::default(),
    }
}

pub fn handle_local_command(
    state: &mut AppState,
    command: LocalCommand,
) -> Result<bool, &'static str> {
    match command {
        LocalCommand::CloseActiveTab => match state.session_manager.active_id() {
            Some(id) => {
                state.session_manager.close(id)?;
                Ok(state.session_manager.active_id().is_none())
            }
            None => Ok(true),
        },
        LocalCommand::NextTab | LocalCommand::PrevTab => Ok(false),
    }
}