use assert_cmd::Command;
use std::process::Command as StdCommand;

const TEST_DATA_PATH: &str = "tests/data";

#[test]
fn test_fortune_flag_m() {
    // Get reference implementation output
    let mut pattern = "apple";
    let ref_output = StdCommand::new("fortune")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");
    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    // Get our implementation output
    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");
    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    // Compare the two outputs
    assert_eq!(
        my_stdout.matches("\n%").count(),
        1,
        "`fortune -m {} {}` - expected: 1 matched quote, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        my_stderr.matches("\n%").count(),
        1,
        "`fortune -m {} {}` - expected: 1 matched file, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -m {} {}` - [stdout], expected: {}, got: {}",
        pattern, TEST_DATA_PATH, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune -m {} {}` - [stderr], expected: {}, got: {}",
        pattern, TEST_DATA_PATH, ref_stderr, my_stderr
    );

    // ref: exit(find_matches() != 0);
    // Should fail if matches are found
    pattern = "apple";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .failure();
    // Should pass if no matches are found
    pattern = "notfound";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .success();
}

#[test]
fn test_fortune_flag_i_and_m() {
    let pwd = std::env::current_dir().unwrap();
    println!("PWD: {:?}", pwd);
    let mut pattern = "apple";
    // Get reference implementation output
    let ref_output = StdCommand::new("fortune")
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");
    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    // Get our implementation output
    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");
    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    // Compare the two outputs
    assert_eq!(
        my_stdout.matches("\n%").count(),
        5,
        "`fortune -i -m {} {}` - expected: 1 matched quote, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        my_stderr.matches("\n%").count(),
        1,
        "`fortune -i -m {} {}` - expected: 1 matched file, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -i -m {} {}` - [stdout], expected: {}, got: {}",
        pattern, TEST_DATA_PATH, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune -i -m {} {}` - [stderr], expected: {}, got: {}",
        pattern, TEST_DATA_PATH, ref_stderr, my_stderr
    );

    pattern = "the";
    // Get reference implementation output
    let ref_output = StdCommand::new("fortune")
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");
    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    // Get our implementation output
    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");
    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    // Compare the two outputs
    assert_eq!(
        my_stdout.matches("\n%").count(),
        3,
        "`fortune -i -m {} {}` - expected: 1 matched quote, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        my_stderr.matches("\n%").count(),
        2,
        "`fortune -i -m {} {}` - expected: 1 matched file, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -i -m {} {}` - [stdout], expected: {}, got: {}",
        pattern, TEST_DATA_PATH, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune -i -m {} {}` - [stderr], expected: {}, got: {}",
        pattern, TEST_DATA_PATH, ref_stderr, my_stderr
    );

    // ref: exit(find_matches() != 0);
    // Should fail if matches are found
    pattern = "apple";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .failure();
    // Should pass if no matches are found
    pattern = "notfound";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .success();
}

#[test]
fn test_fortune_flag_l_and_n() {
    let length = 70;
    let ref_output = StdCommand::new("fortune")
        .arg("-l")
        .arg("-n")
        .arg(length.to_string())
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-l")
        .arg("-n")
        .arg(length.to_string())
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -l -n {} {}` - [stdout], expected: {}, got: {}",
        length, TEST_DATA_PATH, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune -l -n {} {}` - [stderr], expected: {}, got: {}",
        length, TEST_DATA_PATH, ref_stderr, my_stderr
    );
}

#[test]
fn test_fortune_flag_s_and_n() {
    let length = 19;
    let ref_output = StdCommand::new("fortune")
        .arg("-s")
        .arg("-n")
        .arg(length.to_string())
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-s")
        .arg("-n")
        .arg(length.to_string())
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -s -n {} {}` - [stdout], expected: {}, got: {}",
        length, TEST_DATA_PATH, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune -s -n {} {}` - [stderr], expected: {}, got: {}",
        length, TEST_DATA_PATH, ref_stderr, my_stderr
    );
}

#[test]
fn test_fortune_flag_f() {
    let ref_output = StdCommand::new("fortune")
        .arg("-f")
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-f")
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -f {}` - [stdout], expected: {}, got: {}",
        TEST_DATA_PATH, ref_stdout, my_stdout
    );
    //  entry may in any order
    // parse the probability list: probability, path
    // eg.
    //
    // 100.00% tests/data
    //     45.45% apple
    //     9.09% one
    //    45.45% orange
    //     0.00% zero
    //
    let ref_lines: Vec<&str> = ref_stderr.lines().collect::<Vec<&str>>();
    for ref_line in ref_lines {
        assert!(
            my_stderr.contains(ref_line),
            "expected: {}, got: {}",
            ref_line,
            my_stdout
        );
    }
}

#[test]
fn test_fortune_flag_f_and_e() {
    let ref_output = StdCommand::new("fortune")
        .arg("-f")
        .arg("-e")
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-f")
        .arg("-e")
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune -f -e {}` - [stdout], expected: {}, got: {}",
        TEST_DATA_PATH, ref_stdout, my_stdout
    );
    //  entry may in any order
    // parse the probability list: probability, path
    // eg.
    //
    // 100.00% tests/data
    //     45.45% apple
    //     9.09% one
    //    45.45% orange
    //     0.00% zero
    //
    let ref_lines: Vec<&str> = ref_stderr.lines().collect::<Vec<&str>>();
    for ref_line in ref_lines {
        assert!(
            my_stderr.contains(ref_line),
            "fortune -f -e {} - expected: {}, got: {}",
            TEST_DATA_PATH,
            ref_line,
            my_stderr
        );
    }
}
