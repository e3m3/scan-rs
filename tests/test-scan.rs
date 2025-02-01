// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#[cfg(test)]
mod tests {
    use std::env;
    use std::fmt;
    use std::process::Command;
    use std::str;

    const BIN: &str = "scan";

    fn get_command() -> Command {
        let os_name = get_os();
        let shell = get_shell(&os_name);
        Command::new(&shell)
    }

    fn get_os() -> String {
        String::from(if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            ""
        })
    }

    fn get_root_path() -> Result<String, String> {
        match env::var("CARGO_MANIFEST_DIR") {
            Ok(p) => Ok(p),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_shell(os_name: &String) -> String {
        String::from(match os_name.as_str() {
            "linux" => "bash",
            "macos" => "bash",
            "windows" => "cmd",
            _ => {
                eprintln!("Unexpected target_os");
                assert!(false);
                ""
            }
        })
    }

    fn run_test<T: fmt::Display + fmt::Debug>(impl_id: i8, v: &[T]) -> Result<String, String> {
        let str_n = v.len().to_string();
        let str_v = to_string_vec(v);
        let root_path = get_root_path()?;
        let str_in = format!(
            "{}/target/debug/{} {} {} {}",
            root_path,
            BIN,
            impl_id.to_string(),
            str_n,
            str_v.join(" ")
        );
        let output = get_command()
            .arg("-c")
            .arg(str_in)
            .output()
            .expect(&format!("Failed to run test for v={:?}", v));
        if output.status.success() {
            let stdout: &[u8] = output.stdout.as_slice();
            Ok(str::from_utf8(stdout).unwrap_or_default().to_string())
        } else {
            let stderr: &[u8] = output.stderr.as_slice();
            Err(str::from_utf8(stderr).unwrap_or_default().to_string())
        }
    }

    fn to_string_vec<T: fmt::Display>(v: &[T]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn test_body(impl_id: i8, impl_str: &str, v_in: &[i64], v_out: &[i64]) {
        let res = match run_test(impl_id, v_in) {
            Ok(s) => s,
            Err(m) => {
                eprintln!("{}", m);
                assert!(false);
                String::new()
            }
        };
        let str_out = format!("out : {:?}", v_out);
        eprintln!(":: TEST ({})", impl_str);
        eprintln!("Output:\n{}", res);
        eprintln!("Expected:\n{}", str_out);
        eprintln!();
        assert!(res.contains(&str_out))
    }

    #[test]
    fn test_simd_naive_2x_n8_1() {
        let v_in: Vec<i64> = vec![3, 1, 7, 0, 4, 1, 6, 3];
        let v_out: Vec<i64> = vec![0, 3, 4, 11, 11, 15, 16, 22];
        test_body(3, "ScanSimdNaive2x", &v_in, &v_out);
    }

    #[test]
    fn test_seq_n8_1() {
        let v_in: Vec<i64> = vec![3, 1, 7, 0, 4, 1, 6, 3];
        let v_out: Vec<i64> = vec![0, 3, 4, 11, 11, 15, 16, 22];
        test_body(0, "ScanSeq", &v_in, &v_out);
    }

    #[test]
    fn test_seq_naive_n8_1() {
        let v_in: Vec<i64> = vec![3, 1, 7, 0, 4, 1, 6, 3];
        let v_out: Vec<i64> = vec![0, 3, 4, 11, 11, 15, 16, 22];
        test_body(1, "ScanSeqNaive", &v_in, &v_out);
    }

    #[test]
    fn test_seq_naive_2x_n8_1() {
        let v_in: Vec<i64> = vec![3, 1, 7, 0, 4, 1, 6, 3];
        let v_out: Vec<i64> = vec![0, 3, 4, 11, 11, 15, 16, 22];
        test_body(2, "ScanSeqNaive2x", &v_in, &v_out);
    }
}
