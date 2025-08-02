use chrono::{Datelike as _, Weekday};
use iocraft::prelude::*;

fn get_weekday_chinese(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "MON",
        Weekday::Tue => "TUE",
        Weekday::Wed => "WED",
        Weekday::Thu => "THU",
        Weekday::Fri => "FRI",
        Weekday::Sat => "SAT",
        Weekday::Sun => "SUN",
    }
}

#[component]
pub fn MiddleLayout(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut current_time = hooks.use_state(chrono::Local::now);

    hooks.use_future(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            *current_time.write() = chrono::Local::now();
        }
    });

    let time = current_time.get();
    let date_time_str = time.format("%Y-%m-%d %H:%M:%S").to_string();
    let weekday_str = get_weekday_chinese(time.weekday());

    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        ) {
            Text(
                content: date_time_str,
                color: Color::Cyan,
                weight: Weight::Bold
            )
            Text(
                content: weekday_str,
                color: Color::Yellow,
                weight: Weight::Bold
            )
        }
    }
}
