use chrono::{NaiveDate, Duration};
use std::fs::File;
use crate::extractheshtable::extract_link_id_hashtable;
use crate::utilityfun::unzip_file;
use crate::utilityfun::MyCsvChunkReader;
pub fn process_month(
    dbf_path: &str,
    output_path: &str,
    target_path: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_date = start_date;

    while current_date <= end_date {
        // 날짜 기반 파일 이름 생성
        let filename = format!("{}_5Min", current_date.format("%Y%m%d"));
        println!("처리 중인 파일 이름: {}", filename);

        let link_id_map = extract_link_id_hashtable(dbf_path)?;
        println!("link_id_map size = {}", link_id_map.len());

        let chunk_size: usize = 30000;
        unzip_file(&filename, target_path, output_path)?;
        println!("Estimated memory usage: {} bytes", chunk_size * std::mem::size_of::<usize>());

        let input_csv_path = {
            let mut csv_reader = MyCsvChunkReader::<File>::new_from_file(
                target_path.to_string(),
                output_path.to_string(),
                filename.clone(),
                chunk_size.clone(),
                link_id_map.clone(),
            )?;
            csv_reader.run()?
        };

        // CSV 파일 삭제
        if std::fs::metadata(&input_csv_path).is_ok() {
            std::fs::remove_file(&input_csv_path)?;
            println!("입력 CSV 파일 '{}' 삭제 완료.", input_csv_path);
        }

        current_date += Duration::days(1);
    }

    Ok(())
}