리눅스에서 curl https://sh.rustup.rs -sSf | sh -s 명령어로 rust 설치 후 사용
/src/main.rs 파일의 9~11번 라인
    let dbf_path = "/home/ssy/git/INNER_CIRCLE/seoul/filtered_links.dbf".to_string(); 참조할 링크정보가 있는 파일 해당 파일에 있는 링크만 검색해서 저장함 3번째 열에 링크 번호가 있어야함
    let output_path = "/home/ssy//extract_its_data".to_string(); 추줄한 데이터를 저장할 경로
    let target_path = "/home/ssy//its_data".to_string(); 원본 데이터가 위치한 경로
16~17번 라인
            let start_date = NaiveDate::from_ymd(2024, month, 1); 추출할 파일의 시작 년도와 월
            let end_date = if month == 12  추출할 파일의 종료 월 시작파일과 같은년도만 지원 됨
경로 수정후 사용
cargo build --release 명령어 실행 후 /target/release/extract_traffic_infrom 실행
자동으로 1개 월당 1개의 스레드를 할당하니 CPU 여유가 있는 환경에서만 구동하세요


윈도우는 https://rust-kr.org/pages/install/ 링크 참조하여 러스트 환경 구성 후 사용하세요 
