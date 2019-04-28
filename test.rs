use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Once;

static START: Once = Once::new();

pub fn setup() {
    let mut clang = Command::new("make");
    let status = clang.status().expect("Failed to compile!");
    let ok = status.success();
    assert!(ok);
    if !ok {
        panic!("End...");
    }
}

fn run_test(text: &'static str) -> (bool, String, String) {
    START.call_once(|| {
        setup();
    });

    let mut child = Command::new("./main.bin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(text.as_bytes())
            .expect("Failed to write to stdin");
    }

    let out = child.wait_with_output().expect("Failed to read stdout");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn test_del() {
    let commands = "
        Add 2017-06-01 1st of June
        Add 2017-07-08 8th of July
        Add 2017-07-08 Someone's birthday
        Del date == 2017-07-08
    ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out.trim(), "Removed 2 entries");
}

#[test]
fn test_print() {
    let commands = "
        Add 2017-01-01 Holiday
        Add 2017-03-08 Holiday
        Add 2017-1-1 New Year
        Add 2017-1-1 New Year
        Print
    ";
    let expect = "2017-01-01 Holiday\n\
                  2017-01-01 New Year\n\
                  2017-03-08 Holiday\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_find() {
    let commands = r#"
        Add 2017-01-01 Holiday
        Add 2017-03-08 Holiday
        Add 2017-01-01 New Year
        Find event != "working day"
        Add 2017-05-09 Holiday
        "#;
    let expect = "2017-01-01 Holiday\n\
                  2017-01-01 New Year\n\
                  2017-03-08 Holiday\n\
                  Found 3 entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_find_complex() {
    let commands = r#"
        Add 2017-11-21 Tuesday
        Add 2017-11-20 Monday
        Add 2017-11-21 Weekly meeting
        Print
        Find event != "Weekly meeting"
        Find (date > 2017-11-20 AND date < 2017-11-30) AND event != "Tuesday"
        Find (date > 2017-11-20 OR date < 2017-11-30) AND event != "Tuesday"
        Add 2017-11-30 Weekly's meeting
        Del (date >= 2017-11-21) AND event != "Weekly's meeting"
        Print
        Last 2017-11-19
        Find
        Del
        Print
        Find
        "#;

    let expect = "\
                  2017-11-20 Monday\n\
                  2017-11-21 Tuesday\n\
                  2017-11-21 Weekly meeting\n\
                  2017-11-20 Monday\n\
                  2017-11-21 Tuesday\n\
                  Found 2 entries\n\
                  2017-11-21 Weekly meeting\n\
                  Found 1 entries\n\
                  2017-11-20 Monday\n\
                  2017-11-21 Weekly meeting\n\
                  Found 2 entries\n\
                  Removed 2 entries\n\
                  2017-11-20 Monday\n\
                  2017-11-30 Weekly's meeting\n\
                  No entries\n\
                  2017-11-20 Monday\n\
                  2017-11-30 Weekly's meeting\n\
                  Found 2 entries\n\
                  Removed 2 entries\n\
                  Found 0 entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_last() {
    let commands = "
        Add 2017-03-08 Holiday
        Add 2017-03-08 WAT?
        Add 2017-01-01 Holiday
        Add 2017-01-01 New Year
        Last 2016-12-31
        Last 2017-01-01
        Last 2017-06-01
        Add 2017-05-09 Holiday
        Add 2017-05-10 Holiday
        Find date <= 2017-05-09
        Add 0-1-1  Ghr \n\
        Find date <= 0-1-1
        ";
    let expect = "\
                  No entries\n\
                  2017-01-01 New Year\n\
                  2017-03-08 WAT?\n\
                  2017-01-01 Holiday\n\
                  2017-01-01 New Year\n\
                  2017-03-08 Holiday\n\
                  2017-03-08 WAT?\n\
                  2017-05-09 Holiday\n\
                  Found 5 entries\n\
                  0000-01-01 Ghr \n\
                  Found 1 entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect, "\n{} != \n{}, err => \n{}", out, expect, err);

    let commands = r#"
        Add 2017-11-21 Tuesday
        Add 2017-11-20 Monday
        Add 2017-11-21 Weekly meeting
        Print
        Find event != "Weekly meeting"
        Last 2017-11-30
        Del date > 2017-11-20
        Last 2017-11-30
        Last 2017-11-01
        "#;

    let expect = "2017-11-20 Monday\n\
                  2017-11-21 Tuesday\n\
                  2017-11-21 Weekly meeting\n\
                  2017-11-20 Monday\n\
                  2017-11-21 Tuesday\n\
                  Found 2 entries\n\
                  2017-11-21 Weekly meeting\n\
                  Removed 2 entries\n\
                  2017-11-20 Monday\n\
                  No entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect, "\n{} != \n{}, err => \n{}", out, expect, err);
}

#[test]
fn test_some() {
    let commands = r#"
        Add 2017-03-08 Holiday
        Last 2016-12-31
        Last 2017-03-08
        Last 2017-03-07
        Del
        Last 2016-12-31
        "#;
    let expect = "\
                  No entries\n\
                  2017-03-08 Holiday\n\
                  No entries\n\
                  Removed 1 entries\n\
                  No entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_imposible() {
    let commands = r#"
        Add 2017-01-08 one
        Add 2017-02-08 two two
        Add 2018-02-08 three three three
        Add 2018-02-08 z three three three
        Add 2018-02-08 a three three three
        Add 2019-01-08 four four four four
        Add 2019-02-07 five five five five five
        Add 2019-03-06 six six six six six six
        Find date != 2019-03-06
        Print
        Del
        "#;
    let expect = "\
                  2017-01-08 one\n\
                  2017-02-08 two two\n\
                  2018-02-08 three three three\n\
                  2018-02-08 z three three three\n\
                  2018-02-08 a three three three\n\
                  2019-01-08 four four four four\n\
                  2019-02-07 five five five five five\n\
                  Found 7 entries\n\
                  2017-01-08 one\n\
                  2017-02-08 two two\n\
                  2018-02-08 three three three\n\
                  2018-02-08 z three three three\n\
                  2018-02-08 a three three three\n\
                  2019-01-08 four four four four\n\
                  2019-02-07 five five five five five\n\
                  2019-03-06 six six six six six six\n\
                  Removed 8 entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_medved() {
    let commands = r#"
        Add 2018-03-08 preved
        Add 2018-03-08 medved
        Del event != "medved"
        Add 2018-03-08 krasavcheg
        Last 2018-03-08
        Add 2018-03-08 medved
        Last 2018-03-08
    "#;

    let expect = "\
                  Removed 1 entries\n\
                  2018-03-08 krasavcheg\n\
                  2018-03-08 krasavcheg\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_empty() {
    let commands = r#"
        Add 2018-03-08
        Last 2018-03-08
    "#;

    let expect = "\
                  No entries\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect);
}

#[test]
fn test_ge() {
    let commands = r#"
        Add 2019-04-28 a
        Add 2019-04-28 b
        Add 2019-04-28 aa
        Add 2019-04-28
        Del event >= "aa"
        Last 2019-04-28
        Add 2019-04-28 aa
        Last 2019-04-28
    "#;

    let expect = "\
                  Removed 2 entries\n\
                  2019-04-28 a\n\
                  2019-04-28 aa\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect, "\n{} != \n{}, err => \n{}", out, expect, err);
}

#[test]
fn test_geb() {
    let commands = r#"
        Add 2019-04-28 c
        Add 2019-04-28 b
        Add 2019-04-28 cc
        Add 2019-04-28
        Del event >= "cc"
        Last 2019-04-28
        Add 2019-04-28 cc
        Last 2019-04-28
    "#;

    let expect = "\
                  Removed 1 entries\n\
                  2019-04-28 b\n\
                  2019-04-28 cc\n\
                  ";
    let (ok, out, err) = run_test(commands);
    assert!(ok, err);
    assert_eq!(out, expect, "\n{} != \n{}, err => \n{}", out, expect, err);
}
