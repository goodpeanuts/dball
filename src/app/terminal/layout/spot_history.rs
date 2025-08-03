use dball_client::models::Spot;
use iocraft::prelude::*;

use crate::terminal::{
    component::spot::SpotComponent,
    ipc::{RpcResult, send_rpc_request},
};

#[derive(Clone)]
enum HistoryState {
    Init,
    Loading,
    Loaded(Result<Vec<Spot>, String>),
}

#[component]
pub fn SpotHistoryLayout(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut state = hooks.use_state(|| HistoryState::Init);

    // Load prized spots data handler
    let mut load_prized_spots = hooks.use_async_handler(move |_: ()| async move {
        state.set(HistoryState::Loading);
        log::debug!("Loading prized spots data...");
        match send_rpc_request::<Result<Vec<Spot>, String>>(
            dball_client::ipc::RpcService::GetPrizedSpots,
        )
        .await
        {
            Ok(Ok(spots)) => {
                log::debug!("Successfully fetched {} prized spots", spots.len());
                state.set(HistoryState::Loaded(Ok(spots)));
            }
            Err(e) | Ok(Err(e)) => {
                log::error!("Failed to fetch prized spots: {e}");
                state.set(HistoryState::Loaded(Err(e)));
            }
        }
    });

    // Update all unprize spots handler
    let mut update_spots = hooks.use_async_handler({
        let mut state = state;
        move |_: ()| async move {
            state.set(HistoryState::Loading);
            log::info!("Updating all unprize spots...");
            match send_rpc_request::<RpcResult<Vec<Spot>>>(
                dball_client::ipc::RpcService::UpdateAllUnprizeSpots,
            )
            .await
            {
                Ok(Ok(updated_spots)) => {
                    log::info!(
                        "Successfully updated spots, got {} spots back",
                        updated_spots.len()
                    );
                    state.set(HistoryState::Loaded(Ok(updated_spots)));
                }
                Err(e) | Ok(Err(e)) => {
                    log::error!("Failed to update spots: {e}");
                    state.set(HistoryState::Loaded(Err(e)));
                }
            }
        }
    });

    // Initial load
    if matches!(*state.read(), HistoryState::Init) {
        load_prized_spots(());
    }

    // Handle terminal events
    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    // Press U to update all unprize spots
                    KeyCode::Char('u' | 'U') => {
                        update_spots(());
                    }
                    // Press R to refresh/reload prized spots
                    KeyCode::Char('r' | 'R') => {
                        load_prized_spots(());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    let content_elements = match &*state.read() {
        HistoryState::Loaded(Ok(spots)) => {
            if spots.is_empty() {
                vec![
                    element! {
                        Text(content: "No history spots", color: Color::White, weight: Weight::Bold)
                    }
                    .into(),
                ]
            } else {
                spots
                    .iter()
                    .map(|spot| {
                        element! {
                            SpotComponent(value: spot.clone(), has_focus: false)
                        }
                        .into()
                    })
                    .collect::<Vec<_>>()
            }
        }
        HistoryState::Loaded(Err(error)) => {
            vec![
                element! {
                    Text(content: format!("Error: {error}"), color: Color::Red, weight: Weight::Bold)
                }
                .into(),
            ]
        }
        HistoryState::Loading => {
            vec![
                element! {
                    Text(content: "Loading...", color: Color::Yellow, weight: Weight::Bold)
                }
                .into(),
            ]
        }
        HistoryState::Init => {
            vec![
                element! {
                    Text(content: "Initializing...", color: Color::DarkGrey, weight: Weight::Bold)
                }
                .into(),
            ]
        }
    };

    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "Spot History", color: Color::Cyan, weight: Weight::Bold)
            Text(content: "Press U to update all unprize spots\nPress R to refresh", color: Color::Yellow)
            View(
                margin_top: 1,
                flex_direction: FlexDirection::Column,
            ) {
                Fragment(children: content_elements)
            }
        }
    }
}
