use iocraft::prelude::*;

/// 状态面板组件
#[component]
pub fn StatusPanel(_hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "Status", color: Color::Yellow, weight: Weight::Bold)
            Text(content: "Regenerating", color: Color::Cyan)
            Text(content: "Generated", color: Color::Green)
        }
    }
}

/// 时间和期号面板组件
#[component]
pub fn TimeAndPeriodPanel(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let time = hooks.use_state(chrono::Local::now);

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

/// 生成部分面板组件
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

/// 评估结果面板组件
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

/// 日志输出面板组件
#[component]
pub fn LogPanel(_hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let logs = [
        "[2025-07-30 16:30:45] INFO: Starting to fetch latest lottery results",
        "[2025-07-30 16:30:46] INFO: Successfully fetched period 2025084 data",
        "[2025-07-30 16:30:46] INFO: Data validation passed",
        "[2025-07-30 16:30:47] INFO: Starting to generate betting numbers",
        "[2025-07-30 16:30:47] INFO: Using BlueMorn strategy for generation",
        "[2025-07-30 16:30:48] INFO: Number generation completed",
        "[2025-07-30 16:30:48] INFO: Evaluating last period betting results",
        "[2025-07-30 16:30:49] WARN: Last period no win, loss 2 yuan",
        "[2025-07-30 16:30:49] INFO: Preparing to submit next period bet",
        "[2025-07-30 16:30:50] INFO: System running normally",
    ];

    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::White,
            padding: 1,
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "Log Output", color: Color::Yellow, weight: Weight::Bold)
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1,
            ) {
                Text(content: logs[0], color: Color::Green)
                Text(content: logs[1], color: Color::Green)
                Text(content: logs[2], color: Color::Green)
                Text(content: logs[3], color: Color::Green)
                Text(content: logs[4], color: Color::Green)
                Text(content: logs[5], color: Color::Green)
                Text(content: logs[6], color: Color::Green)
                Text(content: logs[7], color: Color::Yellow)
                Text(content: logs[8], color: Color::Green)
                Text(content: logs[9], color: Color::Green)
            }
        }
    }
}
