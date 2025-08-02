use dball_combora::dball::{DBall, DBallBatch};
use iocraft::prelude::*;

use crate::terminal::ipc::send_rpc_request;

pub(crate) mod dball;
pub(crate) mod spot;

/// Status panel component
#[component]
pub fn StatusPanel(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut latest_unprize_spots = hooks.use_state(|| DBallBatch(vec![]));

    hooks.use_future(async move {
        match send_rpc_request::<Result<Vec<DBall>, String>>(
            dball_client::ipc::RpcService::GetUnprizeSpots,
        )
        .await
        {
            Ok(Ok(spots)) => {
                log::info!("Latest unprized spots fetched successfully {spots:?}");
                *latest_unprize_spots.write() = DBallBatch(spots);
            }
            Err(e) | Ok(Err(e)) => {
                log::error!("Failed to fetch latest unprized spots: {e}");
            }
        }
    });

    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "Status", color: Color::Yellow, weight: Weight::Bold)
            Text(content: "Regenerating", color: Color::Cyan)
            Text(content: "Generated", color: Color::Green)
            Text(content: "Unprize Spots", color: Color::White)
            Text(content: format!("{}", *latest_unprize_spots.read()), color: Color::White)
        }
    }
}

/// Time and period panel component
#[component]
pub fn TimeAndPeriodPanel(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut time = hooks.use_state(chrono::Local::now);

    hooks.use_future(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            *time.write() = chrono::Local::now();
        }
    });

    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
        ) {
            Text(content: "Time", color: Color::Yellow, weight: Weight::Bold)
            Text(content: time.get().format("%Y-%m-%d %H:%M:%S").to_string(), color: Color::White)
            View(margin_top: 1) {
                Text(content: "Period", color: Color::Yellow, weight: Weight::Bold)
            }
            Text(content: "2025084", color: Color::Cyan, weight: Weight::Bold)
        }
    }
}

/// Generation part panel component
#[component]
pub fn GenerationPanel(_hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "Generation", color: Color::Yellow, weight: Weight::Bold)
            View(margin_top: 1) {
                Text(content: "Red Balls: 03 08 15 22 28 33", color: Color::Red)
            }
            Text(content: "Blue Ball: 12", color: Color::Blue)
            View(margin_top: 1) {
                Text(content: "Multiplier: 1", color: Color::White)
            }
            View(margin_top: 1) {
                Text(content: "")
            }
            Text(content: "Strategy: BlueMorn", color: Color::Cyan)
            Text(content: "Confidence: 85%", color: Color::Green)
            Text(content: "Expected Return: 2.3x", color: Color::Yellow)
        }
    }
}

/// Evaluation result panel component
#[component]
pub fn EvaluationPanel(_hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "Last Period Evaluation", color: Color::Yellow, weight: Weight::Bold)
            View(margin_top: 1) {
                Text(content: "Period: 2025083", color: Color::White)
            }
            Text(content: "Winning: 05 12 18 25 31 33 + 08", color: Color::Cyan)
            View(margin_top: 1) {
                Text(content: "Bet: 03 08 15 22 28 33 + 12", color: Color::White)
            }
            Text(content: "Result: No Win", color: Color::Red)
            Text(content: "Prize: ¥0", color: Color::Red)
            Text(content: "Cost: ¥2", color: Color::White)
            Text(content: "Net Income: -¥2", color: Color::Red)
        }
    }
}
