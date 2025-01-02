// 파일명: extractheshtable.rs
use std::collections::HashMap;
use std::error::Error;
use dbase::{FieldValue, Reader as DbaseReader};

/// DBF 파일 경로(dbf_path)를 받아
/// LINK_ID만 해시맵으로 추출해 반환하는 함수.
///
/// key: String (LINK_ID)
/// value: bool (단순히 존재 여부를 true로 표시)
pub fn extract_link_id_hashtable(dbf_path: &str) -> Result<HashMap<String, bool>, Box<dyn Error>> {
    // 1) dbase reader 생성
    let mut dreader = DbaseReader::from_path(dbf_path)?;

    // 2) 해시맵
    let mut link_id_map = HashMap::new();

    // 3) 레코드 순회
    for record_result in dreader.iter_records() {
        let record = record_result?; // dbase::Record

        // "LINK_ID" 필드를 가져옴 (Option<&FieldValue>)
        if let Some(field_val) = record.get("LINK_ID") {
            match field_val {
                // 예: Character(Some("1030011501"))
                FieldValue::Character(Some(link_str)) => {
                    // 실제 링크 아이디: link_str
                    link_id_map.insert(link_str.clone(), true);
                },
                FieldValue::Character(None) => {
                    // 빈값인 경우
                    eprintln!("LINK_ID is empty.");
                },
                other => {
                    // 그 외 타입
                    eprintln!("Unexpected LINK_ID field type: {:?}", other);
                }
            }
        } else {
            eprintln!("No LINK_ID field in this record?");
        }
    }

    Ok(link_id_map)
}