#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
use ktts_kma::{WordAnal, analyze, load_kma_dicts};

#[expect(clippy::format_collect, reason = "test helper builds hex strings")]
fn cvc_hex(w: &WordAnal) -> Vec<String> {
    w.morphs
        .iter()
        .map(|m| m.cvc.iter().map(|b| format!("{b:02x}")).collect::<String>())
        .collect()
}

fn pos_str(w: &WordAnal) -> String {
    w.morphs.iter().map(|m| m.pos[0] as char).collect()
}

fn run(ctx: &ktts_kma::KmaContext, text: &str) -> Vec<WordAnal> {
    analyze(ctx, text).unwrap()
}

fn ctx() -> ktts_kma::KmaContext {
    let dic = std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KLangDic");
    load_kma_dicts(&dic).unwrap()
}

#[test]
fn date_with_units() {
    let c = ctx();
    let ws = run(&c, "1995년 10월 17일");
    let pos: Vec<String> = ws.iter().map(pos_str).collect();
    assert_eq!(pos, ["HHH6", "H6", "H6"]);
    let cvc: Vec<Vec<String>> = ws.iter().map(cvc_hex).collect();
    assert_eq!(
        cvc,
        vec![
            vec![
                "100705".to_string(),
                "021401090402".to_string(),
                "0214010b1d130d0d01".to_string(),
                "040b05".to_string(),
            ],
            vec!["0b1d13".to_string(), "0d1509".to_string()],
            vec!["0b1d13101d09".to_string(), "0d1d09".to_string()],
        ]
    );
}

#[test]
fn time_korean_counter() {
    let c = ctx();
    let ws = run(&c, "오늘은 3시 25분입니다");
    let pos: Vec<String> = ws.iter().map(pos_str).collect();
    assert_eq!(pos, ["3]", "H6", "H6c^"]);
    let cvc: Vec<Vec<String>> = ws.iter().map(cvc_hex).collect();
    assert_eq!(
        cvc,
        vec![
            vec!["0d0d01041b09".to_string(), "0d1b05".to_string()],
            vec!["0b0a01".to_string(), "0b1d01".to_string()],
            vec![
                "0d1d010b1d130d0d01".to_string(),
                "091405".to_string(),
                "0d1d01".to_string(),
                "010113041d01050301".to_string(),
            ],
        ]
    );
}

#[test]
fn tele_number() {
    let c = ctx();
    let ws = run(&c, "전화번호 010-1234-5678");
    let pos: Vec<String> = ws.iter().map(pos_str).collect();
    assert_eq!(pos, ["10", "HWM", "H0HHH"]);
    let cvc: Vec<Vec<String>> = ws.iter().map(cvc_hex).collect();
    assert_eq!(
        cvc,
        vec![
            vec!["0e0705140e01".to_string(), "090705140d01".to_string()],
            vec![
                "020d170d1d09020d17".to_string(),
                "0d0a01".to_string(),
                "2c".to_string(),
            ],
            vec![
                "0d1d090d1d010b03110b0301".to_string(),
                "081d010414010b1b01".to_string(),
                "0d0d01100705".to_string(),
                "0d1a02090402".to_string(),
                "101d090b1d13130309".to_string(),
            ],
        ]
    );
}

#[test]
fn percent_unit() {
    let c = ctx();
    let ws = run(&c, "100%");
    assert_eq!(ws.len(), 1);
    assert_eq!(pos_str(&ws[0]), "H6");
    assert_eq!(cvc_hex(&ws[0]), ["090402", "131b01070d01"]);
}

#[test]
fn km_unit() {
    let c = ctx();
    let ws = run(&c, "5km");
    assert_eq!(ws.len(), 1);
    assert_eq!(pos_str(&ws[0]), "H6");
    assert_eq!(cvc_hex(&ws[0]), ["0d0d01", "111d01070d01080a01120701"]);
}

#[test]
fn juche_year() {
    let c = ctx();
    let ws = run(&c, "주체95년");
    assert_eq!(ws.len(), 1);
    assert_eq!(pos_str(&ws[0]), "0H6");
    assert_eq!(
        cvc_hex(&ws[0]),
        ["0e1401100a01", "0214010b1d130d0d01", "040b05"]
    );
}

#[test]
fn juche_date_link() {
    let c = ctx();
    let ws = run(&c, "주체94.10.16-17");
    let pos: Vec<String> = ws.iter().map(pos_str).collect();
    assert_eq!(pos, ["0H6", "H6", "H6W", "H6W"]);
    let cvc: Vec<Vec<String>> = ws.iter().map(cvc_hex).collect();
    assert_eq!(
        cvc,
        vec![
            vec![
                "0e1401100a01".to_string(),
                "0214010b1d130b0301".to_string(),
                "040b05".to_string()
            ],
            vec!["0b1d01".to_string(), "0d1509".to_string()],
            vec![
                "0b1d130d1a02".to_string(),
                "0d1d09".to_string(),
                "091401120701".to_string()
            ],
            vec![
                "0b1d13101d09".to_string(),
                "0d1d09".to_string(),
                "0303010e1d01".to_string()
            ],
        ]
    );
}

#[test]
fn slash_date_special_month() {
    let c = ctx();
    let ws = run(&c, "1995/10/17");
    let pos: Vec<String> = ws.iter().map(pos_str).collect();
    assert_eq!(pos, ["HHH6", "H6", "H6"]);
    let cvc: Vec<Vec<String>> = ws.iter().map(cvc_hex).collect();
    assert_eq!(cvc[1], ["0b1d01", "0d1509"]);
    assert_eq!(cvc[0][0], "100705");
}

#[test]
fn time_hms() {
    let c = ctx();
    let ws = run(&c, "3:25:55");
    let pos: Vec<String> = ws.iter().map(pos_str).collect();
    assert_eq!(pos, ["H6", "H6", "H6"]);
    let cvc: Vec<Vec<String>> = ws.iter().map(cvc_hex).collect();
    assert_eq!(cvc[0], ["0b0a01", "0b1d01"]);
    assert_eq!(cvc[1], ["0d1d010b1d130d0d01", "091405"]);
    assert_eq!(cvc[2], ["0d0d010b1d130d0d01", "100d01"]);
}

#[test]
fn decimal_read() {
    let c = ctx();
    let ws = run(&c, "1.5");
    assert_eq!(ws.len(), 1);
    assert_eq!(pos_str(&ws[0]), "HHH");
    assert_eq!(cvc_hex(&ws[0]), ["0d1d09", "0f0711", "0d0d01"]);
    let ws = run(&c, "0.5");
    assert_eq!(cvc_hex(&ws[0]), ["070b17", "0f0711", "0d0d01"]);
}

#[test]
fn plain_counter_digit() {
    let c = ctx();
    let ws = run(&c, "95년");
    assert_eq!(ws.len(), 1);
    assert_eq!(pos_str(&ws[0]), "H6");
    assert_eq!(cvc_hex(&ws[0]), ["0214010b1d130d0d01", "040b05"]);
}

#[test]
fn order_prefix_digit() {
    let c = ctx();
    let ws = run(&c, "제17차");
    assert_eq!(ws.len(), 1);
    assert_eq!(pos_str(&ws[0]), "4H6");
    assert_eq!(cvc_hex(&ws[0]), ["0e0a01", "0b1d13101d09", "100301"]);
}
