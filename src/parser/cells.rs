use crate::model::page::BTreePageType;

// From https://www.sqlite.org/fileformat.html#b_tree_pages:
// U = usable size of a database page (total page size - reserved space) or (page_size - reserved_bytes)
// P = payload size
// X = max amount of payload that can be stored without overflow
// M = min payload must be stored on page before overflow is allowed
// X is U-35 for table btree leaf pages or ((U-12)*64/255)-23 for index pages.
// M is always ((U-12)*32/255)-23.
// Let K be M+((P-M)%(U-4)).
// If P<=X then all P bytes of payload are stored directly on the btree page without overflow.
// If P>X and K<=X then the first K bytes of P are stored on the btree page and the remaining P-K bytes are stored on overflow pages.
// If P>X and K>X then the first M bytes of P are stored on the btree page and the remaining P-M bytes are stored on overflow pages.
pub fn calculate_amount_payload_on_page(
    usable_page_size: u64,
    payload_size: u64,
    page_type: BTreePageType,
) -> u64 {
    if page_type == BTreePageType::TableInterior {
        assert_eq!(
            payload_size, 0,
            "Table B-Tree interior pages should have no payload"
        );
        return payload_size;
    }

    let max_payload_without_overflow: u64;
    match page_type {
        BTreePageType::TableLeaf => {
            max_payload_without_overflow = usable_page_size - 35;
        }
        _ => {
            max_payload_without_overflow = ((usable_page_size - 12) * 64 / 255) - 23;
        }
    }

    if payload_size <= max_payload_without_overflow {
        return payload_size;
    }

    let min_payload_stored_on_page = ((usable_page_size - 12) * 32 / 255) - 23;
    let k = min_payload_stored_on_page
        + ((payload_size - min_payload_stored_on_page) % (usable_page_size - 4));
    if k <= max_payload_without_overflow {
        return k;
    }
    min_payload_stored_on_page
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: u64 = 4096;

    struct Case {
        page_type: BTreePageType,
        p: u64,
        expected: u64,
    }

    #[test]
    fn overflow_examples() {
        let cases = vec![
            Case {
                page_type: BTreePageType::TableInterior,
                p: 0,
                expected: 0,
            },
            Case {
                page_type: BTreePageType::TableLeaf,
                p: 4000,
                expected: 4000,
            },
            Case {
                page_type: BTreePageType::TableLeaf,
                p: 4062,
                expected: 489,
            },
            Case {
                page_type: BTreePageType::TableLeaf,
                p: 4582,
                expected: 490,
            },
            Case {
                page_type: BTreePageType::IndexInterior,
                p: 900,
                expected: 900,
            },
            Case {
                page_type: BTreePageType::IndexInterior,
                p: 1003,
                expected: 489,
            },
            Case {
                page_type: BTreePageType::IndexInterior,
                p: 4582,
                expected: 490,
            },
            Case {
                page_type: BTreePageType::IndexLeaf,
                p: 900,
                expected: 900,
            },
            Case {
                page_type: BTreePageType::IndexLeaf,
                p: 1003,
                expected: 489,
            },
            Case {
                page_type: BTreePageType::IndexLeaf,
                p: 4582,
                expected: 490,
            },
        ];

        for case in cases {
            let result = calculate_amount_payload_on_page(U, case.p, case.page_type);
            assert_eq!(
                result, case.expected,
                "failed for {:?} with P={}",
                case.page_type, case.p
            );
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn compute_m(u: u64) -> u64 {
        ((u - 12) * 32 / 255) - 23
    }

    fn compute_x(u: u64, page_type: BTreePageType) -> u64 {
        match page_type {
            BTreePageType::TableLeaf => u - 35,
            BTreePageType::IndexLeaf | BTreePageType::IndexInterior => ((u - 12) * 64 / 255) - 23,
            BTreePageType::TableInterior => 0,
        }
    }

    proptest! {
        #[test]
        fn payload_invariants_hold(
            u in 512u64..8192,
            p in 0u64..20000,
            page_type in prop_oneof![
                Just(BTreePageType::TableLeaf),
                Just(BTreePageType::TableInterior),
                Just(BTreePageType::IndexLeaf),
                Just(BTreePageType::IndexInterior),
            ]
        ) {
            prop_assume!(u > 100);

            let p = if page_type == BTreePageType::TableInterior { 0 } else { p };
            let actual = calculate_amount_payload_on_page(u, p, page_type);

            match page_type {
                BTreePageType::TableInterior => {
                    prop_assert_eq!(actual, 0);
                }

                _ => {
                    let m = compute_m(u);
                    let x = compute_x(u, page_type);

                    // Never store more than payload
                    prop_assert!(actual <= p);

                    if p <= x {
                        // Entire payload stored
                        prop_assert_eq!(actual, p);
                    } else {
                        // Overflow case
                        prop_assert!(actual >= m);
                        prop_assert!(actual <= x);
                    }
                }
            }
        }
    }

    #[test]
    fn boundary_cases() {
        let u = 4096;

        for page_type in [
            BTreePageType::TableLeaf,
            BTreePageType::IndexLeaf,
            BTreePageType::IndexInterior,
        ] {
            let m = compute_m(u);
            let x = compute_x(u, page_type);

            // P == X
            assert_eq!(calculate_amount_payload_on_page(u, x, page_type), x);

            // P == X + 1
            let result = calculate_amount_payload_on_page(u, x + 1, page_type);
            assert!(result >= m);
            assert!(result <= x);

            // P == M
            assert_eq!(calculate_amount_payload_on_page(u, m, page_type), m);

            // P == M - 1 (if valid)
            if m > 0 {
                let result = calculate_amount_payload_on_page(u, m - 1, page_type);
                assert_eq!(result, m - 1);
            }

            // Large payload
            let result = calculate_amount_payload_on_page(u, 50_000, page_type);
            assert!(result >= m);
            assert!(result <= x);
        }

        // Table interior always zero
        assert_eq!(
            calculate_amount_payload_on_page(u, 0, BTreePageType::TableInterior),
            0
        );
    }
}
