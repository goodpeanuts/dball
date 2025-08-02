use iocraft::prelude::*;

#[component]
pub fn OpenStatusLayout(_hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "open status", color: Color::Cyan)
        }
    }
}
