use dball_client::service::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 数据库示例 ===\n");

    // 1. 统计总记录数
    match count_records() {
        Ok(count) => println!("📊 数据库中共有 {} 条记录", count),
        Err(e) => println!("❌ 查询记录数失败: {}", e),
    }

    // 2. 获取最新的5条记录
    println!("\n🔥 最新5条记录:");
    match get_latest_records(5) {
        Ok(records) => {
            for record in records {
                println!(
                    "期号: {}, 日期: {}, 号码: {:?} + {}",
                    record.code,
                    record.kj_date.map_or("未知".to_string(), |d| d.to_string()),
                    record.red_numbers(),
                    record.blue_number().unwrap_or(0)
                );
            }
        }
        Err(e) => println!("❌ 查询最新记录失败: {}", e),
    }

    // 3. 查找包含号码1的记录（前3条）
    println!("\n🎯 包含号码1的记录（前3条）:");
    match find_records_with_number(1) {
        Ok(records) => {
            for (i, record) in records.iter().take(3).enumerate() {
                println!(
                    "{}. 期号: {}, 日期: {}, 全部号码: {:?}",
                    i + 1,
                    record.code,
                    record.kj_date.map_or("未知".to_string(), |d| d.to_string()),
                    record.all_numbers()
                );
            }
        }
        Err(e) => println!("❌ 查询包含号码1的记录失败: {}", e),
    }

    // 4. 获取回报最高的记录
    println!("\n回报最高的记录:");
    match get_max_jackpot_record() {
        Ok(record) => {
            println!(
                "期号: {}, 日期: {}, 回报: {} 元, 号码: {:?} + {}",
                record.code,
                record.kj_date.map_or("未知".to_string(), |d| d.to_string()),
                record.jackpot.unwrap_or(0),
                record.red_numbers(),
                record.blue_number().unwrap_or(0)
            );
        }
        Err(e) => println!("❌ 查询回报最高记录失败: {}", e),
    }

    // 5. 查询特定记录
    println!("\n🔍 查询特定期号 '2003001':");
    match get_record_by_code("2003001") {
        Ok(record) => {
            println!(
                "期号: {}, 日期: {}, 星期: {}, 号码: {:?} + {}",
                record.code,
                record.kj_date.map_or("未知".to_string(), |d| d.to_string()),
                record.xq.as_deref().unwrap_or("未知"),
                record.red_numbers(),
                record.blue_number().unwrap_or(0)
            );

            // 测试JSON解析
            match record.parse_json_numbers() {
                Ok(json_numbers) => println!("JSON号码: {:?}", json_numbers),
                Err(e) => println!("JSON解析失败: {}", e),
            }
        }
        Err(e) => println!("❌ 查询特定记录失败: {}", e),
    }

    println!("\n✅ 查询完成！");
    Ok(())
}
