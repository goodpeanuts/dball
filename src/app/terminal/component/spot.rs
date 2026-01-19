use dball_client::models::Spot;
use iocraft::prelude::*;

#[derive(Props)]
pub struct SpotProps {
    pub value: Spot,
    pub has_focus: bool,
}

impl Default for SpotProps {
    fn default() -> Self {
        use chrono::Utc;
        let now = Utc::now().naive_utc();
        Self {
            value: Spot {
                id: Some(1),
                period: "[default]".to_owned(),
                red1: 1,
                red2: 2,
                red3: 3,
                red4: 4,
                red5: 5,
                red6: 6,
                blue: 1,
                magnification: 1,
                prize_status: Some(0),
                deprecated: false,
                created_time: now,
                modified_time: now,
            },
            has_focus: false,
        }
    }
}

#[component]
pub fn SpotComponent(_hooks: Hooks<'_, '_>, props: &SpotProps) -> impl Into<AnyElement<'static>> {
    let spot = &props.value;

    let red_balls = [
        spot.red1, spot.red2, spot.red3, spot.red4, spot.red5, spot.red6,
    ];
    let red_balls_str = red_balls
        .iter()
        .map(|&ball| format!("{ball:02}"))
        .collect::<Vec<_>>()
        .join(",");

    let blue_ball_str = format!("{:02}", spot.blue);

    let multiplier_str = format!("×{}", spot.magnification);

    let (status_text, status_color) = spot_status(spot);

    element! {
        View(
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        ) {
            Text(content: format!("[{}]", spot.period), color: Color::Cyan)
            Text(content: " ", color: Color::White)
            Text(content: red_balls_str, color: Color::Red, weight: Weight::Bold)
            Text(content: "+", color: Color::White)
            Text(content: blue_ball_str, color: Color::Blue, weight: Weight::Bold)
            Text(content: " ", color: Color::White)
            Text(content: multiplier_str, color: Color::Yellow)
            Text(content: " - ", color: Color::White)
            Text(content: status_text, color: status_color, weight: Weight::Bold)
        }
    }
}

pub(crate) fn spot_status(spot: &Spot) -> (String, Color) {
    if let Some(prize_status) = spot.prize_status {
        if prize_status > 0 {
            (
                format!("hit#{prize_status}"),
                if spot.deprecated {
                    Color::DarkMagenta
                } else {
                    Color::Red
                },
            )
        } else {
            (
                "non-prize".to_owned(),
                if spot.deprecated {
                    Color::White
                } else {
                    Color::Cyan
                },
            )
        }
    } else {
        ("pending".to_owned(), Color::Yellow)
    }
}
