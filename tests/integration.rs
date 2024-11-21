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

    pattern = "apple";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .success();
    // Should pass if no matches are found
    pattern = "notfound";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .failure();
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
        4,
        "`fortune -i -m {} {}` - expected: 3 matched quote, got: {}",
        pattern,
        TEST_DATA_PATH,
        my_stdout.split("\n%\n").count()
    );
    assert_eq!(
        my_stderr.matches("\n%").count(),
        2,
        "`fortune -i -m {} {}` - expected: 2 matched file, got: {}",
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

    pattern = "apple";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .success();
    // Should pass if no matches are found
    pattern = "notfound";
    Command::cargo_bin("fortune")
        .unwrap()
        .arg("-i")
        .arg("-m")
        .arg(pattern)
        .arg(TEST_DATA_PATH)
        .assert()
        .failure();
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

#[test]
fn test_fortune_flag_c_and_o() {
    let output = Command::cargo_bin("fortune")
        .unwrap()
        .arg("-c")
        .arg("-o")
        .arg(TEST_DATA_PATH)
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    let first_line = my_stdout.lines().next().unwrap();
    let second_line = my_stdout.lines().nth(1).unwrap();
    let third_line = my_stdout.lines().nth(2).unwrap();

    assert_eq!(
        first_line, "(off/offensive)",
        "`fortune -c -o {}` - expected first_line: (off/offensive), got: {}",
        TEST_DATA_PATH, first_line
    );
    assert_eq!(
        second_line, "%",
        "`fortune -c -o {}` - expected second_line: (on/offensive), got: {}",
        TEST_DATA_PATH, second_line
    );
    assert_eq!(
        third_line, "this is offensive quote.",
        "`fortune -c -o {}` - expected third_line: (on/non-offensive), got: {}",
        TEST_DATA_PATH, third_line
    );
    assert_eq!(
        my_stderr, "",
        "`fortune -c -o {}` - expected stderr: empty, got: {}",
        TEST_DATA_PATH, my_stderr
    );
}

#[test]
fn test_fortune_flag_probs() {
    let testcases = [
        (
            "-f 60% tests/data 40% tests/data2",
            [
                "60.00% tests/data",
                "27.27% apple",
                "5.45% one",
                "27.27% orange",
                "0.00% zero",
                "40.00% tests/data2",
                "20.00% cat",
                "20.00% dog",
            ],
        ),
        (
            "-f 30% tests/data 70% tests/data2",
            [
                "30.00% tests/data",
                "13.64% apple",
                "2.73% one",
                "13.64% orange",
                "0.00% zero",
                "70.00% tests/data2",
                "35.00% cat",
                "35.00% dog",
            ],
        ),
        (
            "-f -e 20% tests/data 80% tests/data2",
            [
                "20.00% tests/data",
                "5.00% apple",
                "5.00% one",
                "5.00% orange",
                "5.00% zero",
                "80.00% tests/data2",
                "40.00% cat",
                "40.00% dog",
            ],
        ),
        (
            "-f tests/data tests/data2",
            [
                "52.38% tests/data",
                "23.81% apple",
                "4.76% one",
                "23.81% orange",
                "0.00% zero",
                "47.62% tests/data2",
                "23.81% cat",
                "23.81% dog",
            ],
        ),
        (
            "-f -e tests/data tests/data2",
            [
                "66.67% tests/data",
                "16.67% apple",
                "16.67% one",
                "16.67% orange",
                "16.67% zero",
                "33.33% tests/data2",
                "16.67% cat",
                "16.67% dog",
            ],
        ),
    ];

    for (input, expected) in testcases.iter() {
        let output = Command::cargo_bin("fortune")
            .unwrap()
            .args(input.split_whitespace().collect::<Vec<&str>>())
            .output()
            .expect("msg: failed to execute our implementation");

        // let my_stdout = String::from_utf8(output.stdout).unwrap();
        let my_stderr = String::from_utf8(output.stderr).unwrap();

        // println!("my_stdout: {}", my_stdout);
        println!("my_stderr: {}", my_stderr);
        let my_lines: Vec<&str> = my_stderr.lines().map(|l| l.trim()).collect::<Vec<&str>>();
        for expected_line in expected.iter() {
            assert!(
                my_lines.contains(expected_line),
                "expected: {}, got: {}",
                expected_line,
                my_stderr
            );
        }
    }
}
