use dball_client::models::Spot;
use iocraft::prelude::*;

use crate::terminal::{component::spot::SpotComponent, ipc::send_rpc_request};

#[derive(Clone)]
enum SpotsState {
    Init,
    Loading,
    Loaded(Result<Vec<Spot>, String>),
}

#[component]
pub fn NextGenLayout(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut state = hooks.use_state(|| SpotsState::Init);

    // Load spots data handler
    let mut load_spots = hooks.use_async_handler(move |_: ()| async move {
        state.set(SpotsState::Loading);
        log::debug!("Loading spots data...");
        match send_rpc_request::<Result<Vec<Spot>, String>>(
            dball_client::ipc::RpcService::GetUnprizeSpots,
        )
        .await
        {
            Ok(Ok(spots)) => {
                log::debug!("Successfully fetched {} unprized spots", spots.len());
                state.set(SpotsState::Loaded(Ok(spots)));
            }
            Err(e) | Ok(Err(e)) => {
                log::error!("Failed to fetch unprized spots: {e}");
                state.set(SpotsState::Loaded(Err(e)));
            }
        }
    });

    // Generate batch spots handler
    let mut generate_spots = hooks.use_async_handler({
        let mut state = state;
        move |_: ()| async move {
            state.set(SpotsState::Loading);
            log::debug!("Generating new batch spots...");
            match send_rpc_request::<Result<(), String>>(
                dball_client::ipc::RpcService::GenerateBatchSpots,
            )
            .await
            {
                Ok(Ok(_)) => {
                    log::info!("Successfully generated new batch spots, refreshing...");
                    // Reload spots after generation
                    match send_rpc_request::<Result<Vec<Spot>, String>>(
                        dball_client::ipc::RpcService::GetUnprizeSpots,
                    )
                    .await
                    {
                        Ok(Ok(spots)) => {
                            log::debug!(
                                "Refreshed after generation: fetched {} spots",
                                spots.len()
                            );
                            state.set(SpotsState::Loaded(Ok(spots)));
                        }
                        Err(e) | Ok(Err(e)) => {
                            log::error!("Failed to refresh after generation: {e}");
                            state.set(SpotsState::Loaded(Err(e)));
                        }
                    }
                }
                Err(e) | Ok(Err(e)) => {
                    log::error!("Failed to generate batch spots: {e}");
                    state.set(SpotsState::Loaded(Err(e)));
                }
            }
        }
    });

    // Deprecate last batch spots handler
    let mut deprecate_spots = hooks.use_async_handler({
        let mut state = state;
        move |_: ()| async move {
            state.set(SpotsState::Loading);
            log::info!("Marking last batch spots as deprecated...");
            match send_rpc_request::<Result<usize, String>>(
                dball_client::ipc::RpcService::DeprecatedLastBatchUnprizedSpot,
            )
            .await
            {
                Ok(Ok(count)) => {
                    log::info!("Successfully marked {count} spots as deprecated, refreshing...");
                    // Reload spots after deprecation
                    match send_rpc_request::<Result<Vec<Spot>, String>>(
                        dball_client::ipc::RpcService::GetUnprizeSpots,
                    )
                    .await
                    {
                        Ok(Ok(spots)) => {
                            log::debug!(
                                "Refreshed after deprecation: fetched {} spots",
                                spots.len()
                            );
                            state.set(SpotsState::Loaded(Ok(spots)));
                        }
                        Err(e) | Ok(Err(e)) => {
                            log::error!("Failed to refresh after deprecation: {e}");
                            state.set(SpotsState::Loaded(Err(e)));
                        }
                    }
                }
                Err(e) | Ok(Err(e)) => {
                    log::error!("Failed to mark spots as deprecated: {e}");
                    state.set(SpotsState::Loaded(Err(e)));
                }
            }
        }
    });

    // Initial load
    if matches!(*state.read(), SpotsState::Init) {
        load_spots(());
    }

    // Handle terminal events
    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    // Press G to generate new spots
                    KeyCode::Char('g' | 'G') => {
                        generate_spots(());
                    }
                    // Press D to deprecate last batch spots
                    KeyCode::Char('d' | 'D') => {
                        deprecate_spots(());
                    }
                    // Press R to refresh/reload spots
                    KeyCode::Char('r' | 'R') => {
                        load_spots(());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    let content_elements = match &*state.read() {
        SpotsState::Loaded(Ok(spots)) => {
            if spots.is_empty() {
                vec![
                    element! {
                        Text(content: "No next generation data available", color: Color::Red, weight: Weight::Bold)
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
        SpotsState::Loaded(Err(error)) => {
            vec![
                element! {
                    Text(content: format!("Error: {error}"), color: Color::Red, weight: Weight::Bold)
                }
                .into(),
            ]
        }
        SpotsState::Loading => {
            vec![
                element! {
                    Text(content: "Loading...", color: Color::Yellow, weight: Weight::Bold)
                }
                .into(),
            ]
        }
        SpotsState::Init => {
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
            Text(content: "Next Generation", color: Color::Cyan, weight: Weight::Bold)
            Text(content: "Press G to generate batch spots\nPress D to deprecate last batch\nPress R to refresh", color: Color::Yellow)
            View(
                margin_top: 1,
                flex_direction: FlexDirection::Column,
            ) {
                Fragment(children: content_elements)
            }
        }
    }
}
