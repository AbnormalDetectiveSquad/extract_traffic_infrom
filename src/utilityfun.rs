use std::fs;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::io::{Read, Seek};
use std::array;
use zip::ZipArchive;
use std::error::Error;
use csv::{ReaderBuilder, StringRecord, Reader, WriterBuilder};
use std::collections::HashMap;
// 함수: unzip_file
// 인자:
//   - filename   : &str      -> 확장자 없는 파일 이름 (예: "20240101_5Min")
//   - target_path: &str      -> 압축 파일(zip)이 있는 폴더 경로
//   - output_path: &str      -> 압축을 해제할 폴더(여기에 /work 생성)
// 반환: Result<(), Box<dyn std::error::Error>>
pub struct MyCsvChunkReader<R: Read + Seek>  {
    rdr: Reader<R>,  
    chunk_size: usize,  // 한 번에 몇 줄씩 읽을지
    save_mat:[[String;6]; 30000],
    target_path: String,
    output_path: String,
    filename: String,
    link_id_map: HashMap<String, bool>,
    number: u64,
}
impl<R: Read + Seek> MyCsvChunkReader<R> {
    /// 사용자가 이미 `Reader<R>`를 준비해 넘겨주는 버전
    pub fn new(rdr: Reader<R>, chunk_size: usize) -> Self {
        let save_mat: [[String; 6]; 30000] = array::from_fn(|_| {
            array::from_fn(|_| String::new())
        });
        let link_id_map: HashMap<String, bool> = HashMap::new();
        MyCsvChunkReader { rdr, chunk_size, save_mat, target_path: String::new(), output_path: String::new(), filename: String::new(),link_id_map,number: 0 }
    }

    /// 편의 함수: 파일 전용
    pub fn new_from_file(
        target_path: String,
        output_path: String,
        filename: String,
        chunk_size: usize,
        link_id_map: HashMap<String, bool>
    ) -> Result<MyCsvChunkReader<File>, Box<dyn Error>> {
        let path = format!("{}/work/{}.csv", output_path, filename);
        println!("path = {}",path);
        let file = File::open(path)?;
        let rdr: Reader<File> = ReaderBuilder::new().has_headers(false).from_reader(file);
        let save_mat: [[String; 6]; 30000] = array::from_fn(|_| {
            array::from_fn(|_| String::new())
        });
        let number: u64 = 0;
        Ok(MyCsvChunkReader { rdr, chunk_size, save_mat, target_path, output_path, filename,link_id_map,number})
    }

    pub fn next_chunk(&mut self) -> Option<Result<Vec<StringRecord>, csv::Error>> {
        let mut lines = Vec::with_capacity(self.chunk_size);

        for _ in 0..self.chunk_size {
            match self.rdr.records().next() {
                Some(Ok(rec)) => lines.push(rec),
                Some(Err(e)) => return Some(Err(e)),
                None => break,
            }
        }

        if lines.is_empty() {
            None
        } else {
            Some(Ok(lines))
        }
    }
    pub fn clear_save_mat(&mut self){
        // 우선 save_mat 전체를 ""로 초기화 (원하면 부분 초기화만 해도 됨)
        for row in 0..30000 {
            for col in 0..6 {
                self.save_mat[row][col].clear();
            }
        }
    }
    pub fn fill_mat_from_chunk(&mut self, lines: &[StringRecord]) -> usize {
        self.clear_save_mat();
        let max_row = lines.len().min(30000);  // 최대 30000행까지만
        for row in 0..max_row {
            let rec = &lines[row];
            // 이 행에 있는 필드 개수
            let fields = rec.len().min(6);
            for col in 0..fields {
                self.save_mat[row][col] = rec[col].to_string();
            }
            // 나머지 col..6는 이미 clear()로 ""
        }

        max_row
    }
    /// 예: 필터링 후 save_mat 채우기, 반환값은 "이번에 새로 저장된 수"
    pub fn fill_mat_from_chunk_filtered(
        &mut self,
        lines: &[StringRecord],
        already_matched: usize
    ) -> usize {
        // clear save_mat
        self.clear_save_mat();
        let max_capacity = 30000;
        let needed = self.link_id_map.len()*288;
        let mut row_count = 0;
        let mut total = already_matched;

        for rec in lines {
            //print!("{}번째 줄\n",self.number);
            //self.number += 1;
            // 예시: 2번째 컬럼이 link_id
            if rec.len() < 3 { continue; }

            let link_id = &rec[2];

            if self.link_id_map.contains_key(link_id) {
                // fill one row in save_mat
                let fields = rec.len().min(6);
                for col in 0..fields {
                    self.save_mat[row_count][col] = rec[col].to_string();
                }
                row_count += 1;
                total += 1;
                // 만약 save_mat 꽉찼거나, total >= needed
                if row_count >= max_capacity || total >= needed {
                    break;
                }
            }
        } 

        // return "실제로 이번에 row_count개 저장"
        row_count
    }
    /// 새로 추가할 함수:
    /// `save_mat`에 들어 있는 유효 행을 CSV 파일에 이어쓰기(append)한다.
    /// - "유효 행"을 어떻게 판정할지는 여기선 "행[0]이 빈 문자열인지"로 예시 처리
    /// - 만약 한 행의 0번 열이 ""면 -> 그 아래 행도 읽지 않는다고 가정.
    /// 
    /// 반환:
    ///   None -> `save_mat`에 유효 데이터가 1행도 없음
    ///   Some(Ok(n)) -> n행 썼음
    ///   Some(Err(e)) -> 쓰는 중 CSV 에러
    pub fn save_mat_to_csv(&mut self) -> Option<Result<usize, csv::Error>> {
        let out_path = format!("{}/{}.csv", self.output_path, self.filename);

        // 1) 몇 행이 유효한지 계산
        //   예: 행[0][0]이 빈 문자열이면 유효행0
        //   행[1][0]이 빈 문자열이면 그 아래 행도 안 읽는다는 식
        let mut valid_rows = 0;
        for i in 0..30000 {
            if self.save_mat[i][0].is_empty() {
                // 첫 컬럼이 ""이면 -> 유효 데이터 없다 가정, break
                break;
            }
            valid_rows += 1;
        }

        // 유효행이 없으면 None
        if valid_rows == 0 {
            return None;
        }

        // 2) 파일 append 모드로 열기
        let file_result = OpenOptions::new()
            .create(true)
            .append(true)
            .open(out_path);

        // 만약 파일 열기 실패 -> csv::Error로 래핑
        let file = match file_result {
            Ok(f) => f,
            Err(e) => {
                let io_err = std::io::Error::new(std::io::ErrorKind::Other, e);
                return Some(Err(csv::Error::from(io_err)));
            }
        };

        // 3) CSV Writer
        let mut wtr = WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        // 4) valid_rows만큼 저장
        for i in 0..valid_rows {
            // 한 행: &["foo", "bar", ...] -> wtr.write_record(...)
            // save_mat[i]는 [String; 6]
            // write_record는 IntoIterator<Item = &str>
            // => map each String to &str
            let row_strs = self.save_mat[i].iter().map(|s| s.as_str());
            if let Err(e) = wtr.write_record(row_strs) {
                return Some(Err(e));
            }
        }

        // 5) flush
        if let Err(e) = wtr.flush() {
            return Some(Err(e.into())); 
        }

        // Some(Ok(쓴 행 수))
        Some(Ok(valid_rows))
    }
    

    /// (새) run 메서드: next_chunk() -> fill_mat_from_chunk(...) -> save_mat_to_csv(...) 반복
    ///  - out_path: CSV 출력 경로 (append)
    ///  - 무한 루프 돌다가 EOF면 break
    ///  - return Err(e) => 중간 에러 발생
    pub fn run(&mut self) -> Result<String, csv::Error>  {
        let out_path = format!("{}/{}.csv", self.output_path, self.filename);
        // 1) 만약 기존 파일이 존재한다면 삭제
        //    이를 통해 append 모드가 아닌, 사실상 "새로 쓰기" 효과
        if fs::metadata(&out_path).is_ok() {
            // 파일이 존재 -> 지움
            fs::remove_file(&out_path).map_err(|e| {
                // remove_file의 오류를 csv::Error로 변환
                csv::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
            println!("기존 파일 '{}'을(를) 삭제했습니다.", out_path);
        }
        let needed = self.link_id_map.len()*288;  // 목표 개수
        let mut total_matched = 0;       // 지금까지 필터링 성공한 수

        loop {
            // 1) chunk_size줄 읽기
            let maybe_chunk = self.next_chunk();
            let chunk = match maybe_chunk {
                None => {
                    // EOF -> 종료
                    println!("EOF reached. total_matched={}", total_matched);
                    break;
                }
                Some(Ok(lines)) => lines,
                Some(Err(e)) => return Err(e),
            };

            // 2) fill_mat_from_chunk_filtered:
            //    (예) 필터링 후 save_mat에 옮기고, 그 행 수를 반환
            let row_count = self.fill_mat_from_chunk_filtered(&chunk,total_matched);

            // 지금 fill_mat_from_chunk_filtered(...) 예시:
            //   - row_count = 이번 청크에서 새로 저장된 행 수
            //   - 내부에서 total_matched += row_count (또는 반환값으로 받을 수도)
            // 여기서는 단순히 row_count를 리턴한다고 가정
            total_matched += row_count;

            // 3) save_mat -> CSV append
            //    (row_count==0이어도, 아래에서 처리할 수 있음;
            //     다만 row_count==0이면 어차피 "valid_rows=0" → None)
            match self.save_mat_to_csv() {
                None => {
                    // save_mat에 유효행이 없으면 => 이번 청크 실제로 쓴 게 없다는 뜻
                    // => 그래도 다음 청크로 가야할 수도 있으므로 "continue"
                    // or if you want, do nothing special
                    continue;
                }
                Some(Ok(n)) => {
                    println!("Wrote {} rows this chunk, total_matched={}", n, total_matched);
                    // 만약 우리가 필요한 모든 아이디(needed)에 도달했으면 중단
                    if total_matched >= needed {
                        println!("All needed Link IDs found & written!");
                        break;
                    }
                }
                Some(Err(e)) => return Err(e),
            }
        }
        let input_csv_path = format!("{}/work/{}.csv", self.output_path, self.filename);
        Ok(input_csv_path)
    
    }
}


pub fn unzip_file(
    filename: &str,
    target_path: &str,
    output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {

    // 1) .zip 파일 경로 구성
    let zip_file_path = format!("{}/{}.zip", target_path, filename);

    // 2) work 폴더 생성 (예: /home/ssy/extract_its_data/work)
    let work_folder = format!("{}/work", output_path);
    fs::create_dir_all(&work_folder)?;

    // 3) ZIP 파일 열기
    let zip_file = fs::File::open(&zip_file_path)
        .map_err(|e| format!("Failed to open ZIP file '{}': {}", zip_file_path, e))?;

    // 4) zip::ZipArchive로 읽어오기
    let mut archive = ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read ZIP archive '{}': {}", zip_file_path, e))?;

    // 5) 내부 항목을 순회하며 해제
    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)
            .map_err(|e| format!("Error reading file index {}: {}", i, e))?;

        let outpath = PathBuf::from(&work_folder).join(file_in_zip.name());

        // 폴더인지 파일인지 구분
        if file_in_zip.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            // 상위 디렉토리가 없으면 생성
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            // 파일 쓰기
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file_in_zip, &mut outfile)?;
        }

        // 로그(옵션)
        println!("Extracted: {}", outpath.display());
    }

    println!("압축 해제 완료: {}", work_folder);
    Ok(())
}

// 공개 함수: output_path와 filename을 받아서, "output_path/work/filename.csv" 읽기
pub fn print_first_csv_row(output_path: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    // 1) CSV 경로 구성
    //    예: "/home/ssy/extract_its_data/work/20240101_5Min.csv"
    let csv_path: String = format!("{}/work/{}.csv", output_path, filename);

    // 2) CSV Reader
    let mut rdr = ReaderBuilder::new()
        .has_headers(false) // CSV에 헤더 있다고 가정
        .from_path(&csv_path)?; // &csv_path -> &str


    let mut iter = rdr.records();

    // 3) 첫 번째 레코드만 시도
    if let Some(result) = iter.next() {
        let record = result?;
        println!("CSV 파일: {}, 첫 레코드: {:?}", csv_path, record);
    } else {
        println!("CSV 파일: {}, 레코드가 없습니다.", csv_path);
    }

    Ok(())
}