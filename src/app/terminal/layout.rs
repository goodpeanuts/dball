#[expect(clippy::wildcard_imports)]
use super::components::*;
use iocraft::prelude::*;

/// 主布局组件
#[component]
pub fn MainLayout(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();

    // 计算各区域尺寸
    let left_width = width / 3;
    let center_width = width / 3;
    let right_width = width - left_width - center_width;

    // 左列比例：上35%，下65%
    let left_top_height = (height * 35) / 100;
    let left_bottom_height = height - left_top_height;

    // 中列比例：上75%，下25%
    let center_top_height = (height * 75) / 100;
    let center_bottom_height = height - center_top_height;

    // 创建分割线 - 需要考虑padding，所以长度要更短
    let horizontal_divider = "─".repeat((left_width - 4) as usize);

    element! {
        View(
            width,
            height,
            flex_direction: FlexDirection::Row,
            background_color: Color::Black,
        ) {
            // 左列：状态 + 分割线 + 评估结果
            View(
                width: left_width,
                height,
                flex_direction: FlexDirection::Column,
                padding_left: 1,
                padding_right: 1,
            ) {
                // 上半部分 (35%)
                View(height: left_top_height - 1, padding: 1) {
                    StatusPanel()
                }
                // 分割线
                View(height: 1, padding_left: 1) {
                    Text(content: horizontal_divider.clone(), color: Color::White)
                }
                // 下半部分 (65%)
                View(height: left_bottom_height, padding: 1) {
                    EvaluationPanel()
                }
            }

            // 垂直分割线
            View(
                width: 1,
                height,
                background_color: Color::White,
            )

            // 中列：时间期号 + 分割线 + 生成部分
            View(
                width: center_width - 1,
                height,
                flex_direction: FlexDirection::Column,
                padding_left: 1,
                padding_right: 1,
            ) {
                // 上半部分 (75%)
                View(height: center_top_height - 1, padding: 1) {
                    TimeAndPeriodPanel()
                }
                // 分割线
                View(height: 1, padding_left: 1) {
                    Text(content: horizontal_divider.clone(), color: Color::White)
                }
                // 下半部分 (25%)
                View(height: center_bottom_height, padding: 1) {
                    GenerationPanel()
                }
            }

            // 右列：日志输出
            View(
                width: right_width,
                height,
                padding: 1,
            ) {
                LogPanel()
            }
        }
    }
}
