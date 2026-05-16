// JSON 출력 및 에러 종료 헬퍼.
use serde_json::Value;

/// JSON 을 stdout 으로 한 줄 출력한다. broken-pipe 는 무시한다.
pub fn print_json(value: Value) {
    use std::io::Write;
    let _ = writeln!(std::io::stdout(), "{value}");
}

/// 에러 JSON 을 stderr 로 출력하고 1 로 종료한다.
pub fn print_error(message: &str) -> ! {
    let obj = serde_json::json!({ "error": message });
    eprintln!("{obj}");
    // FFI 없음. stdout flush 안전을 위해 _exit 사용.
    unsafe { libc::_exit(1) }
}

/// stdin 전체를 String 으로 읽는다.
pub fn read_stdin() -> Result<String, String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_serializes_value() {
        // 실제 stdout 캡처는 통합 테스트에서. 여기서는 직렬화만 검증.
        let v = serde_json::json!({"a": 1});
        assert_eq!(v.to_string(), r#"{"a":1}"#);
        // 실행해도 패닉 없어야 한다.
        print_json(v);
    }
}
