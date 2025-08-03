use iocraft::prelude::*;

mod logs;
mod middle;
mod nextgen;
mod open_status;
mod spot_history;

pub(crate) use logs::init_logger;

/// Main layout component
#[component]
pub fn MainLayout(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    const LEFT_WIDTH: u16 = 52;

    let (width, height) = hooks.use_terminal_size();

    // Ensure enough space for display, reserve 1 line each for top and bottom
    let usable_height = height.saturating_sub(2);

    let left_width = LEFT_WIDTH;
    let remaining_width = width.saturating_sub(left_width);
    let center_width = remaining_width / 2;
    let right_width = remaining_width - center_width;

    // Left column ratio: 45% top, 55% bottom (optimized ratio to avoid being too tall)
    let left_top_height = (usable_height * 45) / 100;
    let left_bottom_height = usable_height - left_top_height;

    // Center column ratio: 70% top, 30% bottom (adjusted ratio to avoid being too tall)
    let center_top_height = (usable_height * 70) / 100;
    let center_bottom_height = usable_height - center_top_height;

    element! {
        View(
            width,
            height,
            flex_direction: FlexDirection::Row,
            background_color: Color::Black,
            padding: 1,
        ) {
            // Left column: NextGen + SpotHistory (dynamic width)
            View(
                width: left_width.saturating_sub(1),
                height: usable_height,
                flex_direction: FlexDirection::Column,
                margin_right: 1,
            ) {
                // NextGen area
                View(
                    height: left_top_height.saturating_sub(1),
                    border_style: BorderStyle::Round,
                    border_color: Color::Blue,
                    background_color: Color::Black,
                    margin_bottom: 1,
                    padding: 1,
                ) {
                    nextgen::NextGenLayout()
                }

                // SpotHistory area
                View(
                    height: left_bottom_height,
                    border_style: BorderStyle::Round,
                    border_color: Color::Green,
                    background_color: Color::Black,
                    padding: 1,
                ) {
                    spot_history::SpotHistoryLayout()
                }
            }

            // Center column: OpenStatus + Middle
            View(
                width: center_width.saturating_sub(1),
                height: usable_height,
                flex_direction: FlexDirection::Column,
                margin_right: 1,
            ) {
                // OpenStatus area
                View(
                    height: center_top_height.saturating_sub(1),
                    border_style: BorderStyle::Round,
                    border_color: Color::Yellow,
                    background_color: Color::Black,
                    margin_bottom: 1,
                    padding: 1,
                ) {
                    open_status::OpenStatusLayout()
                }

                // Middle area
                View(
                    height: center_bottom_height,
                    border_style: BorderStyle::Round,
                    border_color: Color::Magenta,
                    background_color: Color::Black,
                    padding: 1,
                ) {
                    middle::MiddleLayout()
                }
            }

            // Right column: log output (remove duplicate border)
            View(
                width: right_width,
                height: usable_height,
                border_style: BorderStyle::Round,
                border_color: Color::White,
                background_color: Color::Black,
                flex_direction: FlexDirection::Column,
                padding: 1,
            ) {
                logs::LogsLayout()
            }
        }
    }
}
