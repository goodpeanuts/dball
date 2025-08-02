use dball_combora::dball::DBall;
use iocraft::prelude::*;

#[derive(Props)]
pub struct DBallProps {
    pub value: DBall,
    #[expect(unused)]
    pub has_focus: bool,
}

#[component]
pub fn DBallComponent(_hooks: Hooks<'_, '_>, props: &DBallProps) -> impl Into<AnyElement<'static>> {
    let dball = &props.value;

    let red_balls_str = dball
        .rball
        .iter()
        .map(|&ball| format!("{ball:02}"))
        .collect::<Vec<_>>()
        .join(" ");

    let blue_ball_str = format!("{:02}", dball.bball);

    let multiplier_str = format!("×{}", dball.magnification);

    element! {
        View(
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        ) {
            Text(content: red_balls_str, color: Color::Red, weight: Weight::Bold)
            Text(content: " ", color: Color::White)
            Text(content: blue_ball_str, color: Color::Blue, weight: Weight::Bold)
            Text(content: " ", color: Color::White)
            Text(content: multiplier_str, color: Color::Yellow)
        }
    }
}
