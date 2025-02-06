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

    const N8_1_IN: [i64; 8] = [3, 1, 7, 0, 4, 1, 6, 3];
    const N8_1_OUT: [i64; 8] = [0, 3, 4, 11, 11, 15, 16, 22];

    const N15_1_IN: [i64; 15] = [18, 12, 18, 0, 19, 10, 7, 17, 0, 1, 8, 17, 18, 17, 9];
    const N15_1_OUT: [i64; 15] = [
        0, 18, 30, 48, 48, 67, 77, 84, 101, 101, 102, 110, 127, 145, 162,
    ];

    const N16_1_IN: [i64; 16] = [2, 2, 4, 8, 15, 12, 4, 19, 8, 11, 15, 12, 9, 17, 14, 15];
    const N16_1_OUT: [i64; 16] = [
        0, 2, 4, 8, 16, 31, 43, 47, 66, 74, 85, 100, 112, 121, 138, 152,
    ];

    const N100_1_IN: [i64; 100] = [
        0, 13, 6, 18, 19, 9, 3, 8, 2, 6, 12, 13, 7, 2, 9, 17, 8, 9, 0, 14, 5, 18, 10, 12, 5, 16, 2,
        10, 5, 5, 13, 8, 12, 18, 1, 3, 2, 10, 13, 9, 11, 19, 2, 2, 18, 12, 2, 9, 14, 9, 0, 8, 14,
        15, 16, 2, 7, 2, 2, 15, 13, 3, 11, 16, 7, 15, 15, 20, 1, 10, 18, 13, 1, 4, 18, 8, 19, 3, 8,
        20, 3, 9, 14, 4, 20, 11, 0, 8, 6, 4, 3, 19, 3, 18, 13, 0, 2, 13, 11, 11,
    ];
    const N100_1_OUT: [i64; 100] = [
        0, 0, 13, 19, 37, 56, 65, 68, 76, 78, 84, 96, 109, 116, 118, 127, 144, 152, 161, 161, 175,
        180, 198, 208, 220, 225, 241, 243, 253, 258, 263, 276, 284, 296, 314, 315, 318, 320, 330,
        343, 352, 363, 382, 384, 386, 404, 416, 418, 427, 441, 450, 450, 458, 472, 487, 503, 505,
        512, 514, 516, 531, 544, 547, 558, 574, 581, 596, 611, 631, 632, 642, 660, 673, 674, 678,
        696, 704, 723, 726, 734, 754, 757, 766, 780, 784, 804, 815, 815, 823, 829, 833, 836, 855,
        858, 876, 889, 889, 891, 904, 915,
    ];

    #[test]
    fn test_par_cpu_naive_2x_n8_1() {
        test_body(4, "ScanParCPUNaive2x", &N8_1_IN, &N8_1_OUT);
    }

    #[test]
    fn test_par_cpu_naive_2x_n15_1() {
        test_body(4, "ScanParCPUNaive2x", &N15_1_IN, &N15_1_OUT);
    }

    #[test]
    fn test_par_cpu_naive_2x_n16_1() {
        test_body(4, "ScanParCPUNaive2x", &N16_1_IN, &N16_1_OUT);
    }

    #[test]
    fn test_par_cpu_naive_2x_n100_1() {
        test_body(4, "ScanParCPUNaive2x", &N100_1_IN, &N100_1_OUT);
    }

    #[test]
    fn test_seq_n8_1() {
        test_body(0, "ScanSeq", &N8_1_IN, &N8_1_OUT);
    }

    #[test]
    fn test_seq_n15_1() {
        test_body(0, "ScanSeq", &N15_1_IN, &N15_1_OUT);
    }

    #[test]
    fn test_seq_n16_1() {
        test_body(0, "ScanSeq", &N16_1_IN, &N16_1_OUT);
    }

    #[test]
    fn test_seq_n100_1() {
        test_body(0, "ScanSeq", &N100_1_IN, &N100_1_OUT);
    }

    #[test]
    fn test_seq_naive_n8_1() {
        test_body(1, "ScanSeqNaive", &N8_1_IN, &N8_1_OUT);
    }

    #[test]
    fn test_seq_naive_n15_1() {
        test_body(1, "ScanSeqNaive", &N15_1_IN, &N15_1_OUT);
    }

    #[test]
    fn test_seq_naive_n16_1() {
        test_body(1, "ScanSeqNaive", &N16_1_IN, &N16_1_OUT);
    }

    #[test]
    fn test_seq_naive_n100_1() {
        test_body(1, "ScanSeqNaive", &N100_1_IN, &N100_1_OUT);
    }

    #[test]
    fn test_seq_naive_2x_n8_1() {
        test_body(2, "ScanSeqNaive2x", &N8_1_IN, &N8_1_OUT);
    }

    #[test]
    fn test_seq_naive_2x_n15_1() {
        test_body(2, "ScanSeqNaive2x", &N15_1_IN, &N15_1_OUT);
    }

    #[test]
    fn test_seq_naive_2x_n16_1() {
        test_body(2, "ScanSeqNaive2x", &N16_1_IN, &N16_1_OUT);
    }

    #[test]
    fn test_seq_naive_2x_n100_1() {
        test_body(2, "ScanSeqNaive2x", &N100_1_IN, &N100_1_OUT);
    }

    #[test]
    fn test_simd_naive_2x_n8_1() {
        test_body(3, "ScanSimdNaive2x", &N8_1_IN, &N8_1_OUT);
    }

    #[test]
    fn test_simd_naive_2x_n15_1() {
        test_body(3, "ScanSimdNaive2x", &N15_1_IN, &N15_1_OUT);
    }

    #[test]
    fn test_simd_naive_2x_n16_1() {
        test_body(3, "ScanSimdNaive2x", &N16_1_IN, &N16_1_OUT);
    }

    #[test]
    fn test_simd_naive_2x_n100_1() {
        test_body(3, "ScanSimdNaive2x", &N100_1_IN, &N100_1_OUT);
    }
}
