// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#[cfg(test)]
mod tests {
    use test_scan::test_body;
    use test_scan::N100_1_IN;
    use test_scan::N100_1_OUT;
    use test_scan::N15_1_IN;
    use test_scan::N15_1_OUT;
    use test_scan::N16_1_IN;
    use test_scan::N16_1_OUT;
    use test_scan::N8_1_IN;
    use test_scan::N8_1_OUT;

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
}
