리눅스에서 curl https://sh.rustup.rs -sSf | sh -s 명령어로 rust 설치 후 사용
/src/main.rs 파일의 9~11번 라인
    let dbf_path = "/home/ssy/git/INNER_CIRCLE/seoul/filtered_links.dbf".to_string(); 참조할 링크정보가 있는 파일 해당 파일에 있는 링크만 검색해서 저장함 3번째 열에 링크 번호가 있어야함
    let output_path = "/home/ssy//extract_its_data".to_string(); 추줄한 데이터를 저장할 경로
    let target_path = "/home/ssy//its_data".to_string(); 원본 데이터가 위치한 경로
경로 수정후 사용
cargo build --release 명령어 실행 후 /target/release/extract_traffic_infrom 실행

