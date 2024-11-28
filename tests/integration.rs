use assert_cmd::Command;
use ctor::ctor;
use env_logger::Env;
use log::info;
use std::process::Command as StdCommand;

const TEST_DATA_PATH: &str = "tests/data";

#[ctor]
fn setup() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    // Add /usr/games to PATH, which is necessary for Ubuntu/Debian, *BSD systems;
    let current_path = std::env::var("PATH").unwrap();
    if !current_path.contains("/usr/games") {
        let new_path = format!("{}:/usr/games", current_path);
        std::env::set_var("PATH", new_path);
    }
    // info!("[current directory]: {:?}", std::env::current_dir().unwrap());
}

#[test]
fn test_fortune_flag_m() {
    info!(
        "[current directory]: {:?}",
        std::env::current_dir().unwrap()
    );

    let testcases = [("apple", 1, 1), ("the", 3, 1)];

    for (pattern, expected_num_cookies, expected_num_files) in testcases {
        let args = format!("-m {} {}", pattern, TEST_DATA_PATH);
        // Get reference implementation output
        let ref_output = StdCommand::new("fortune")
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .output()
            .expect("msg: failed to execute reference implementation");
        let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
        let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

        // Get our implementation output
        let output = Command::cargo_bin("fortune")
            .unwrap()
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .output()
            .expect("msg: failed to execute our implementation");
        let my_stdout = String::from_utf8(output.stdout).unwrap();
        let my_stderr = String::from_utf8(output.stderr).unwrap();

        // Compare the two outputs
        let my_num_cookies = my_stdout.matches("\n%").count();
        assert_eq!(
            expected_num_cookies, my_num_cookies,
            "`fortune {}` - expected: {} matched cookies, got: {}\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
            args,
            expected_num_cookies, my_num_cookies,
            ref_stdout, my_stdout
        );
        let my_num_files = my_stderr.matches("\n%").count();
        assert_eq!(
            expected_num_files, my_num_files,
            "`fortune {}` - expected: {} matched file, got: {}\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
            args,
            expected_num_files, my_num_files,
            ref_stderr, my_stderr
        );

        assert_eq!(
            ref_stdout, my_stdout,
            "`fortune {}`\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
            args, ref_stdout, my_stdout
        );
        assert_eq!(
            ref_stderr, my_stderr,
            "`fortune {}`\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
            args, ref_stderr, my_stderr
        );
    }

    let testcases = [("apple", true), ("notfound", false)];

    for (patter, result) in testcases {
        let args = format!("-m {} {}", patter, TEST_DATA_PATH);
        let assert_result = Command::cargo_bin("fortune")
            .unwrap()
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .assert();
        if result {
            assert_result.success();
        } else {
            assert_result.failure();
        }
    }
}

#[test]
fn test_fortune_flag_i_and_m() {
    let testcases = [("apple", 5, 1), ("the", 4, 2)];

    for (pattern, expected_num_cookies, expected_num_files) in testcases {
        let args = format!("-i -m {} {}", pattern, TEST_DATA_PATH);
        // Get reference implementation output
        let ref_output = StdCommand::new("fortune")
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .output()
            .expect("msg: failed to execute reference implementation");
        let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
        let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

        // Get our implementation output
        let output = Command::cargo_bin("fortune")
            .unwrap()
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .output()
            .expect("msg: failed to execute our implementation");
        let my_stdout = String::from_utf8(output.stdout).unwrap();
        let my_stderr = String::from_utf8(output.stderr).unwrap();

        // Compare the two outputs
        let my_num_cookies = my_stdout.matches("\n%").count();
        assert_eq!(
            expected_num_cookies, my_num_cookies,
            "`fortune {}` - expected: {} matched cookies, got: {}\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
            args,
            expected_num_cookies, my_num_cookies,
            ref_stdout, my_stdout
        );
        let my_num_files = my_stderr.matches("\n%").count();
        assert_eq!(
            expected_num_files, my_num_files,
            "`fortune {}` - expected: {} matched file, got: {}\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
            args,
            expected_num_files, my_num_files,
            ref_stderr, my_stderr
        );

        assert_eq!(
            ref_stdout, my_stdout,
            "`fortune {}`\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
            args, ref_stdout, my_stdout
        );
        assert_eq!(
            ref_stderr, my_stderr,
            "`fortune {}`\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
            args, ref_stderr, my_stderr
        );
    }

    let testcases = [("apple", true), ("notfound", false)];

    for (patter, result) in testcases {
        let args = format!("-i -m {} {}", patter, TEST_DATA_PATH);
        let assert_result = Command::cargo_bin("fortune")
            .unwrap()
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .assert();
        if result {
            assert_result.success();
        } else {
            assert_result.failure();
        }
    }
}

#[test]
fn test_fortune_flag_l_and_n() {
    let length = 70;
    let args = format!("-l -n {} {}", length, TEST_DATA_PATH);
    let ref_output = StdCommand::new("fortune")
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune {}`\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
        args, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune {}`\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
        args, ref_stderr, my_stderr
    );
}

#[test]
fn test_fortune_flag_s_and_n() {
    let length = 19;
    let args = format!("-s -n {} {}", length, TEST_DATA_PATH);
    let ref_output = StdCommand::new("fortune")
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune {}`\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
        args, ref_stdout, my_stdout
    );
    assert_eq!(
        ref_stderr, my_stderr,
        "`fortune {}`\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
        args, ref_stderr, my_stderr
    );
}

#[test]
fn test_fortune_flag_f() {
    let args = format!("-f {}", TEST_DATA_PATH);
    let ref_output = StdCommand::new("fortune")
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune {}`\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
        args, ref_stdout, my_stdout
    );

    let msg = format!(
        "`fortune {}`\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
        args, ref_stderr, my_stderr
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
    let my_lines: Vec<&str> = my_stderr.lines().collect::<Vec<&str>>();
    for ref_line in ref_lines {
        // some system may use absolute path, so we remove the path prefix if exists
        let ref_parts = ref_line.split(" ").collect::<Vec<&str>>();
        let ref_percentage = ref_parts[0];
        let ref_path = ref_parts[1];

        let mut found = false;
        for my_line in my_lines.iter() {
            let my_parts = my_line.split(" ").collect::<Vec<&str>>();
            let my_percentage = my_parts[0];
            let my_path = my_parts[1];
            if my_percentage == ref_percentage && ref_path.contains(my_path) {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "{}\n cannot find '{}' in '{}'",
            msg, ref_line, my_stdout
        );
    }
}

#[test]
fn test_fortune_flag_f_and_e() {
    let args = format!("-f -e {}", TEST_DATA_PATH);
    let ref_output = StdCommand::new("fortune")
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute reference implementation");

    let ref_stdout = String::from_utf8(ref_output.stdout).unwrap();
    let ref_stderr = String::from_utf8(ref_output.stderr).unwrap();

    let output = Command::cargo_bin("fortune")
        .unwrap()
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        ref_stdout, my_stdout,
        "`fortune {}`\n[ref_stdout]:\n{}\n[my_stdout]:\n{}",
        args, ref_stdout, my_stdout
    );
    let msg = format!(
        "`fortune {}`\n[ref_stderr]:\n{}\n[my_stderr]:\n{}",
        args, ref_stderr, my_stderr
    );

    assert_eq!(
        ref_stdout, my_stdout,
        "{}\n [stdout], expected: {}, got: {}",
        msg, ref_stdout, my_stdout
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
    let my_lines: Vec<&str> = my_stderr.lines().collect::<Vec<&str>>();
    for ref_line in ref_lines {
        // some system may use absolute path, so we remove the path prefix if exists
        let ref_parts = ref_line.split(" ").collect::<Vec<&str>>();
        let ref_percentage = ref_parts[0];
        let ref_path = ref_parts[1];

        let mut found = false;
        for my_line in my_lines.iter() {
            let my_parts = my_line.split(" ").collect::<Vec<&str>>();
            let my_percentage = my_parts[0];
            let my_path = my_parts[1];
            if my_percentage == ref_percentage && ref_path.contains(my_path) {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "{}\n cannot find '{}' in '{}'",
            msg, ref_line, my_stdout
        );
    }
}

#[test]
fn test_fortune_flag_c_and_o() {
    let args = format!("-c -o {}", TEST_DATA_PATH);
    let output = Command::cargo_bin("fortune")
        .unwrap()
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();

    let msg = format!(
        "`fortune {}`\n[my_stdout]:\n{}\n[my_stderr]:\n{}",
        args, my_stdout, my_stderr
    );

    let expected_lines = ["(off/offensive)", "%", "this is offensive cookie."];
    let my_lines: Vec<&str> = my_stdout.lines().collect::<Vec<&str>>();

    for (i, expected_line) in expected_lines.iter().enumerate() {
        assert_eq!(
            *expected_line, my_lines[i],
            "{}\n expected: '{}', got: '{}'",
            msg, expected_line, my_lines[i]
        );
    }
    assert_eq!(
        my_stderr, "",
        "{} - expected stderr: empty, got: {}",
        msg, my_stderr
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

    for (args, expected) in testcases.iter() {
        let output = Command::cargo_bin("fortune")
            .unwrap()
            .args(args.split_whitespace().collect::<Vec<&str>>())
            .output()
            .expect("msg: failed to execute our implementation");

        // let my_stdout = String::from_utf8(output.stdout).unwrap();
        let my_stderr = String::from_utf8(output.stderr).unwrap();

        // Compare the two outputs
        let my_lines: Vec<&str> = my_stderr.lines().map(|l| l.trim()).collect::<Vec<&str>>();
        for expected_line in expected.iter() {
            assert!(
                my_lines.contains(expected_line),
                "`fortune {}`\n[expected_stderr]:\n{}\n[my_stderr]:\n{}",
                args,
                expected.join("\n"),
                my_stderr
            );
        }
    }
}


#[test]
fn test_fortune_embed() {
    let args = "-c";
    let output = Command::cargo_bin("fortune")
        .unwrap()
        .args(args.split_whitespace().collect::<Vec<&str>>())
        .output()
        .expect("msg: failed to execute our implementation");

    let my_stdout = String::from_utf8(output.stdout).unwrap();
    let my_stderr = String::from_utf8(output.stderr).unwrap();
    let msg = format!(
        "`fortune {}`\n[my_stdout]:\n{}\n[my_stderr]:\n{}",
        args, my_stdout, my_stderr
    );
    assert!(my_stdout.len() > 0, "{}\n - expected: non-empty stdout, got: empty", msg);
    // assert!(my_stderr.len() > 0, "{}\n - expected: non-empty stderr, got: empty", msg);
}
