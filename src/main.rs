use chrono::{NaiveDate, Duration};
use std::thread;
mod mounth_module;
mod extractheshtable;
mod utilityfun;

use mounth_module::process_month;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dbf_path = "/home/ssy/git/INNER_CIRCLE/seoul/filtered_links.dbf".to_string();
    let output_path = "/home/ssy//extract_its_data".to_string();
    let target_path = "/home/ssy//its_data".to_string();

    // 월별 시작일과 종료일 설정
    let months: Vec<(NaiveDate, NaiveDate)> = (1..=12)
        .map(|month| {
            let start_date = NaiveDate::from_ymd(2024, month, 1);
            let end_date = if month == 12 {
                NaiveDate::from_ymd(2024, 12, 31)
            } else {
                NaiveDate::from_ymd(2024, month + 1, 1) - Duration::days(1)
            };
            (start_date, end_date)
        })
        .collect();


    // 병렬 실행을 위한 스레드 생성
    let mut handles = Vec::new();
    for (start_date, end_date) in months {
        let dbf_path = dbf_path.clone();
        let output_path = output_path.clone();
        let target_path = target_path.clone();

        // 스택 크기를 32MB로 설정한 스레드 생성
        let handle = thread::Builder::new()
            .stack_size(64 * 1024 * 1024) // 스택 크기 32MB
            .spawn(move || {
                process_month(&dbf_path, &output_path, &target_path, start_date, end_date)
                    .expect("process_month failed");
            })
            .unwrap();

        handles.push(handle);
    }

    // 모든 스레드 종료 대기
    for handle in handles {
        handle.join().unwrap();
    }

    println!("모든 월별 작업 완료!");

    Ok(())
}