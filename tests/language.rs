use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct Output {
    success: bool,
    stdout: Vec<String>,
    stderr: String,
}

fn run(name: &str, source: &str) -> Output {
    let mut path: PathBuf = std::env::temp_dir();
    path.push(format!("redox-test-{}-{}.simple", std::process::id(), name));
    fs::write(&path, source).expect("failed to write test program");

    let output = Command::new(env!("CARGO_BIN_EXE_redox"))
        .arg("-i")
        .arg(&path)
        .output()
        .expect("failed to run the Redox binary");

    let _ = fs::remove_file(&path);

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    // Skip the licence banner, which ends with a blank line.
    let mut lines = stdout.lines();
    let program_stdout = match lines.find(|line| line.is_empty()) {
        Some(_) => lines.map(str::to_string).collect(),
        None => vec![],
    };

    Output {
        success: output.status.success(),
        stdout: program_stdout,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn assert_output(name: &str, source: &str, expected: &[&str]) {
    let output = run(name, source);
    assert!(output.success, "{name} failed: {}", output.stderr);
    let expected: Vec<String> = expected.iter().map(|line| line.to_string()).collect();
    assert_eq!(output.stdout, expected, "{name} produced unexpected output");
}

fn assert_error(name: &str, source: &str, error: &str) {
    let output = run(name, source);
    assert!(!output.success, "{name} unexpectedly succeeded");
    assert!(
        output.stderr.contains(error),
        "{name} reported the wrong error: {}",
        output.stderr
    );
    assert!(
        !output.stderr.contains("panicked"),
        "{name} panicked instead of reporting an error: {}",
        output.stderr
    );
}

fn assert_parse_error(name: &str, source: &str) {
    assert_error(name, source, "Error when constructing AST");
}

#[test]
fn character_literals_are_supported() {
    assert_output(
        "character_literals",
        "Create Character c with value 'x'\nDisplay c\n",
        &["'x'"],
    );
}

#[test]
fn escape_sequences_are_supported_in_character_literals() {
    assert_output(
        "character_escapes",
        "Display '\\''\nDisplay '\\\\'\nCreate Character c with value '\\t'\nDisplay c Equals '\\t'\n",
        &["'''", "'\\'", "True"],
    );
}

#[test]
fn escape_sequences_are_supported_in_text_literals() {
    assert_output(
        "text_escapes",
        "Display \"tab:\\there\"\nDisplay \"new\\nline\"\nDisplay \"quote: \\\"\"\nDisplay \"slash: \\\\\"\n",
        &["tab:\there", "new", "line", "quote: \"", "slash: \\"],
    );
}

#[test]
fn invalid_escape_sequences_are_rejected() {
    assert_parse_error("bad_escape", "Display \"\\q\"\n");
}

#[test]
fn text_displays_without_quotes() {
    assert_output(
        "text_display",
        "Display \"hello\"\nDisplay ['h', 'i']\n",
        &["hello", "['h', 'i']"],
    );
}

#[test]
fn real_literals_accept_signs_and_partial_decimal_points() {
    assert_output(
        "real_literals",
        "Display +1.5\nDisplay 1.\nDisplay .5\nDisplay -.5\n",
        &["1.5", "1", "0.5", "-0.5"],
    );
}

#[test]
fn out_of_range_integer_literal_is_a_compile_time_error() {
    assert_error(
        "integer_range",
        "Display 9223372036854775808\n",
        "IntegerLiteralOutOfRange",
    );
}

#[test]
fn bare_list_declarations_are_rejected() {
    assert_parse_error("bare_list", "Create List a\n");
}

#[test]
fn empty_list_literals_need_a_known_element_type() {
    assert_error("untyped_empty_list", "Display []\n", "UntypedListLiteralError");
}

#[test]
fn empty_lists_can_be_assigned_to_typed_lists() {
    assert_output(
        "empty_list_assignment",
        "Create List<Integer> a with value [1]\n\
         Set a to []\n\
         Display a\n\
         Create List<List<Integer>> b with value [[]]\n\
         Set b to [[]]\n\
         Display b\n",
        &["[]", "[[]]"],
    );
}

#[test]
fn list_literals_unify_numeric_element_types() {
    assert_output(
        "list_literal_unification",
        "Create List<Real> a with value [1, 2.5]\n\
         Display a\n",
        &["[1, 2.5]"],
    );
}

#[test]
fn list_literal_with_incompatible_elements_is_an_error() {
    assert_error(
        "incompatible_list_literal",
        "Create List<Integer> a with value [1, \"two\"]\n",
        "TypeMismatchError",
    );
}

#[test]
fn append_checks_element_types_and_supports_scalars_and_text() {
    assert_output(
        "append_behaviour",
        "Create List<Integer> a with value [1, 2]\n\
         Set a to a Append 3\n\
         Display a\n\
         Create List<Real> b with value [1]\n\
         Set b to b Append 2.5\n\
         Display b\n\
         Create Text t with value \"Hello\"\n\
         Set t to t Append \" \"\n\
         Set t to t Append 'W'\n\
         Display t\n",
        &["[1, 2, 3]", "[1, 2.5]", "Hello W"],
    );
}

#[test]
fn text_and_character_lists_can_be_appended() {
    assert_output(
        "text_and_character_list_append",
        "Create Text t with value \"hi\"\n\
         Set t to t Append ['!', '?']\n\
         Display t\n\
         Create List<Character> cs with value ['a']\n\
         Set cs to cs Append \"bc\"\n\
         Display cs\n",
        &["hi!?", "['a', 'b', 'c']"],
    );
}

#[test]
fn append_rejects_incompatible_lists() {
    assert_error(
        "append_incompatible",
        "Create List<Integer> a with value [1]\n\
         Create List<Text> b with value [\"x\"]\n\
         Set a to a Append b\n",
        "OperatorInputTypeError",
    );
}

#[test]
fn append_binds_tighter_than_equality() {
    assert_output(
        "append_precedence",
        "Create List<Integer> a with value [1]\n\
         Display a Append [2] Equals [1, 2]\n",
        &["True"],
    );
}

#[test]
fn length_returns_the_number_of_elements() {
    assert_output(
        "length",
        "Create List<Integer> a with value [1, 2, 3]\n\
         Create Integer l with value Length(a)\n\
         Display l\n\
         Create List<List<Integer>> nested with value [[], [1]]\n\
         Display Length(nested)\n\
         Set a to []\n\
         Display Length(a) + 1\n",
        &["3", "2", "1"],
    );
}

#[test]
fn length_works_on_text() {
    assert_output(
        "length_text",
        "Create Text t with value \"hello\"\nDisplay Length(t)\nDisplay Length(\"\")\n",
        &["5", "0"],
    );
}

#[test]
fn length_rejects_non_sequences() {
    assert_error("length_type", "Display Length(1)\n", "OperatorInputTypeError");
}

#[test]
fn unary_minus_preserves_integer_type() {
    assert_output(
        "unary_minus",
        "Create Integer a with value -5\n\
         Display a\n\
         Create Real b with value -2.5\n\
         Display b\n",
        &["-5", "-2.5"],
    );
}

#[test]
fn copyof_arguments_are_coerced() {
    assert_output(
        "argument_coercion",
        "Function f takes (Copyof Real a) returns Real {\n\
           Return a\n\
         }\n\
         Create Integer x with value 3\n\
         Display f(x)\n",
        &["3"],
    );
}

#[test]
fn literals_can_be_passed_to_reference_parameters() {
    assert_output(
        "reference_literal",
        "Function bump takes (Integer a) returns Integer {\n\
           Set a to a + 1\n\
           Return a\n\
         }\n\
         Display bump(1)\n",
        &["2"],
    );
}

#[test]
fn reference_arguments_are_not_implicitly_cast() {
    assert_error(
        "reference_cast",
        "Function f takes (Real a) returns Nothing { }\n\
         Create Integer x with value 1\n\
         Run f(x)\n",
        "FunctionArgumentTypeError",
    );
}

#[test]
fn return_values_are_coerced_to_the_declared_type() {
    assert_output(
        "return_coercion",
        "Function f takes (Copyof Integer a) returns Real {\n\
           Return a\n\
         }\n\
         Display f(2)\n",
        &["2"],
    );
}

#[test]
fn wrong_return_type_is_an_error() {
    assert_error(
        "wrong_return_type",
        "Function f takes (Copyof Integer a) returns Integer {\n\
           Return \"hello\"\n\
         }\n\
         Display f(2)\n",
        "FunctionReturnTypeError",
    );
}

#[test]
fn falling_off_the_end_of_a_non_nothing_function_is_an_error() {
    assert_error(
        "missing_return",
        "Function f takes (Copyof Integer a) returns Integer {\n\
           Create Integer x with value a\n\
         }\n\
         Display f(2)\n",
        "FunctionDidNotReturnError",
    );
}

#[test]
fn recursive_functions_work_with_copyof_parameters() {
    assert_output(
        "recursion",
        "Function fact takes (Copyof Integer n) returns Integer {\n\
           If n <= 1 then {\n\
             Return 1\n\
           }\n\
           Return n * fact(n - 1)\n\
         }\n\
         Display fact(5)\n",
        &["120"],
    );
}

#[test]
fn function_bodies_can_access_globals() {
    assert_output(
        "global_access",
        "Create Integer total with value 10\n\
         Function add takes (Integer n) returns Nothing {\n\
           Set total to total + n\n\
         }\n\
         Run add(5)\n\
         Display total\n",
        &["15"],
    );
}

#[test]
fn returning_from_inside_a_block_does_not_leak_scopes() {
    assert_output(
        "return_scope",
        "Function f takes (Copyof Integer x) returns Integer {\n\
           If x Equals 1 then {\n\
             Return 10\n\
           }\n\
           Return 20\n\
         }\n\
         Create Integer a with value 1\n\
         Display f(a)\n\
         Display a\n\
         Display f(a)\n",
        &["10", "1", "10"],
    );
}

#[test]
fn if_then_does_not_require_otherwise() {
    assert_output(
        "if_without_otherwise",
        "Create Integer a with value 3\n\
         If a Equals 3 then {\n\
           Display \"yes\"\n\
         }\n\
         Display \"after\"\n",
        &["yes", "after"],
    );
}

#[test]
fn if_requires_then() {
    assert_parse_error("if_without_then", "If True { }\n");
}

#[test]
fn inline_blocks_are_terminated_by_the_closing_brace() {
    assert_output(
        "inline_block",
        "If True then { Display 1 }\nDisplay 2\n",
        &["1", "2"],
    );
}

#[test]
fn statements_are_terminated_by_newlines() {
    assert_parse_error("newline_sensitivity", "Display 1 Display 2\n");
}

#[test]
fn run_requires_a_function_call() {
    assert_parse_error("run_requires_call", "Run 1\n");
}

#[test]
fn return_in_main_drops_the_value() {
    assert_output("return_in_main", "Display 1\nReturn 5\n", &["1"]);
}

#[test]
fn nothing_literals_are_supported() {
    assert_output(
        "nothing_literal",
        "Create Nothing n with value Nothing\nDisplay n\nDisplay Nothing\n",
        &["Nothing", "Nothing"],
    );
}

#[test]
fn text_can_be_indexed_as_characters() {
    assert_output(
        "text_indexing",
        "Create Text t with value \"hi\"\n\
         Display t[0]\n\
         Display t[1]\n",
        &["'h'", "'i'"],
    );
}

#[test]
fn out_of_bounds_access_is_a_runtime_error() {
    assert_error(
        "out_of_bounds",
        "Create List<Integer> a with value [1]\n\
         Display a[3]\n",
        "IndexOutOfBoundsError",
    );
}

#[test]
fn too_many_arguments_is_an_error_not_a_panic() {
    assert_error(
        "too_many_arguments",
        "Function f takes (Copyof Integer a) returns Integer {\n\
           Return a\n\
         }\n\
         Display f(1, 2)\n",
        "FunctionArgumentNumberError",
    );
}

#[test]
fn boolean_values_display_as_literals() {
    assert_output(
        "boolean_display",
        "Display True\nDisplay False\nDisplay [True, False]\n",
        &["True", "False", "[True, False]"],
    );
}

#[test]
fn integer_division_by_zero_is_a_runtime_error() {
    assert_error("division_by_zero", "Display 1 / 0\n", "DivisionByZeroError");
}

#[test]
fn real_division_by_zero_follows_ieee_754() {
    assert_output(
        "real_division_by_zero",
        "Display 1.0 / 0.0\nDisplay -1.0 / 0.0\nDisplay 0.0 / 0.0\n",
        &["inf", "-inf", "NaN"],
    );
}

#[test]
fn scalar_to_list_casts_respect_the_element_type() {
    assert_output(
        "scalar_to_list_cast",
        "Create List<Text> xs with value \"hi\"\n\
         Display xs\n\
         Create Text t with value ['a', 'b']\n\
         Display t\n\
         Create List<Character> cs with value \"cd\"\n\
         Display cs\n",
        &["[hi]", "ab", "['c', 'd']"],
    );
}

#[test]
fn list_access_binds_tighter_than_multiplication() {
    assert_output(
        "access_precedence",
        "Create List<Integer> a with value [2, 3]\n\
         Display a[0] * a[1]\n\
         Display 2 * a[0]\n\
         Display 1 + 2 * 3\n\
         Display -a[0]\n",
        &["6", "4", "7", "-2"],
    );
}

#[test]
fn statements_run_without_a_trailing_newline() {
    assert_output("no_trailing_newline", "Display 1", &["1"]);
}

#[test]
fn crlf_and_blank_lines_terminate_statements() {
    assert_output("crlf", "Display 1\r\n\r\nDisplay 2\r\n", &["1", "2"]);
}

#[test]
fn while_loops_until_the_condition_is_false() {
    assert_output(
        "while_true",
        "Create Integer i with value 0\n\
         While i < 3 {\n\
           Set i to i + 1\n\
         }\n\
         Display i\n",
        &["3"],
    );
}

#[test]
fn while_runs_while_the_condition_is_false_when_asked_to() {
    assert_output(
        "while_false",
        "Create Integer i with value 0\n\
         While i > 0 is False {\n\
           Set i to i + 1\n\
         }\n\
         Display i\n",
        &["1"],
    );
}

#[test]
fn while_condition_must_be_a_boolean() {
    assert_error(
        "while_non_boolean",
        "While 1 {\n  Display 1\n}\n",
        "ConditionalTypeError",
    );
}

#[test]
fn the_is_clause_requires_a_boolean_literal() {
    assert_parse_error("while_is_literal", "While True is 1 { }\n");
}

#[test]
fn if_condition_must_be_a_boolean() {
    assert_error(
        "if_non_boolean",
        "If 1 then {\n  Display 1\n}\n",
        "ConditionalTypeError",
    );
}

#[test]
fn conditions_may_be_function_calls() {
    assert_output(
        "condition_call",
        "Function is_three takes (Copyof Integer a) returns Boolean {\n\
           Return a Equals 3\n\
         }\n\
         If is_three(3) then {\n\
           Display \"yes\"\n\
         }\n",
        &["yes"],
    );
}

#[test]
fn redefining_a_variable_is_an_error() {
    assert_error(
        "redefine_var",
        "Create Integer a with value 1\nCreate Integer a with value 2\n",
        "RedefineExistingVarError",
    );
}

#[test]
fn duplicate_functions_are_an_error() {
    assert_error(
        "redefine_function",
        "Function f takes (Copyof Integer a) returns Nothing { }\n\
         Function f takes (Copyof Integer a) returns Nothing { }\n",
        "RedefineFunctionError",
    );
}

#[test]
fn duplicate_argument_names_are_an_error() {
    assert_error(
        "duplicate_argument",
        "Function f takes (Copyof Integer a, Copyof Integer a) returns Nothing { }\n",
        "DuplicateArgumentError",
    );
}

#[test]
fn using_an_undefined_variable_is_an_error() {
    assert_error("undefined_var", "Display nope\n", "VarDoesNotExistError");
}

#[test]
fn calling_an_undefined_function_is_an_error() {
    assert_error(
        "undefined_function",
        "Display nope(1)\n",
        "FunctionDoesNotExistError",
    );
}

#[test]
fn reference_parameters_mutate_the_callers_variable() {
    assert_output(
        "reference_mutation",
        "Function double takes (Integer a) returns Nothing {\n\
           Set a to a * 2\n\
         }\n\
         Create Integer x with value 3\n\
         Run double(x)\n\
         Display x\n",
        &["6"],
    );
}

#[test]
fn copyof_parameters_leave_the_callers_variable_alone() {
    assert_output(
        "copyof_mutation",
        "Function double takes (Copyof Integer a) returns Nothing {\n\
           Set a to a * 2\n\
         }\n\
         Create Integer x with value 3\n\
         Run double(x)\n\
         Display x\n",
        &["3"],
    );
}

#[test]
fn reference_parameters_can_mutate_lists() {
    assert_output(
        "reference_list_mutation",
        "Function push takes (List<Integer> a) returns Nothing {\n\
           Set a to a Append 4\n\
         }\n\
         Create List<Integer> xs with value [1]\n\
         Run push(xs)\n\
         Display xs\n",
        &["[1, 4]"],
    );
}

#[test]
fn expressions_can_be_passed_to_reference_parameters() {
    assert_output(
        "reference_expression",
        "Function bump takes (Integer a) returns Integer {\n\
           Set a to a + 1\n\
           Return a\n\
         }\n\
         Create Integer x with value 1\n\
         Display bump(x + 1)\n\
         Display x\n",
        &["3", "1"],
    );
}

#[test]
fn functions_cannot_see_the_callers_locals() {
    assert_error(
        "caller_locals",
        "Function peek takes (Copyof Integer a) returns Integer {\n\
           Return hidden\n\
         }\n\
         Function caller takes (Copyof Integer n) returns Integer {\n\
           Create Integer hidden with value n\n\
           Return peek(n)\n\
         }\n\
         Display caller(1)\n",
        "VarDoesNotExistError",
    );
}

#[test]
fn run_discards_the_return_value_but_keeps_side_effects() {
    assert_output(
        "run_side_effects",
        "Function tick takes (Integer counter) returns Integer {\n\
           Set counter to counter + 1\n\
           Display \"tick\"\n\
           Return 99\n\
         }\n\
         Create Integer n with value 0\n\
         Run tick(n)\n\
         Display n\n",
        &["tick", "1"],
    );
}

#[test]
fn functions_may_return_nothing_with_a_bare_return() {
    assert_output(
        "bare_return_nothing",
        "Function noop takes (Copyof Integer a) returns Nothing {\n\
           Return\n\
         }\n\
         Run noop(1)\n\
         Display \"done\"\n",
        &["done"],
    );
}

#[test]
fn integer_overflow_is_a_runtime_error() {
    assert_error(
        "integer_overflow",
        "Display 9223372036854775807 + 1\n",
        "IntegerOverflowError",
    );
}

#[test]
fn modulo_works_and_modulo_by_zero_is_a_runtime_error() {
    assert_output("modulo", "Display 7 Modulo 2\n", &["1"]);
    assert_error(
        "modulo_zero",
        "Display 1 Modulo 0\n",
        "DivisionByZeroError",
    );
}

#[test]
fn text_access_out_of_bounds_is_a_runtime_error() {
    assert_error(
        "text_out_of_bounds",
        "Create Text t with value \"hi\"\nDisplay t[2]\n",
        "IndexOutOfBoundsError",
    );
}

#[test]
fn negative_indexes_are_runtime_errors() {
    assert_error(
        "negative_index",
        "Create List<Integer> a with value [1]\nDisplay a[-1]\n",
        "IndexOutOfBoundsError",
    );
}

#[test]
fn indexing_a_non_list_is_an_error() {
    assert_error(
        "index_non_list",
        "Create Integer a with value 1\nDisplay a[0]\n",
        "OperatorInputTypeError",
    );
}

#[test]
fn create_without_a_value_uses_the_type_default() {
    assert_output(
        "defaults",
        "Create Integer i\n\
         Create Real r\n\
         Create Text t\n\
         Create Boolean b\n\
         Create Character c\n\
         Display i\n\
         Display r\n\
         Display t Equals \"\"\n\
         Display b\n\
         Display c Equals '\\0'\n",
        &["0", "0", "True", "False", "True"],
    );
}

#[test]
fn list_declarations_default_to_empty() {
    assert_output(
        "list_default",
        "Create List<Integer> a\nDisplay a\nDisplay Length(a)\n",
        &["[]", "0"],
    );
}

#[test]
fn list_equality_compares_elements() {
    assert_output(
        "list_equality",
        "Create List<Integer> a with value [1, 2]\n\
         Create List<Integer> b with value [1, 2]\n\
         Display a Equals b\n\
         Display a NotEquals [1, 2, 3]\n",
        &["True", "True"],
    );
}

#[test]
fn nested_list_types_survive_access() {
    assert_output(
        "nested_access_type",
        "Create List<List<Integer>> a with value [[], [1]]\n\
         Display Length(a[0])\n\
         Create List<Integer> inner with value a[1]\n\
         Display inner\n",
        &["0", "[1]"],
    );
}

#[test]
fn length_of_a_text_and_a_list_agrees_with_indexing() {
    assert_output(
        "length_bounds",
        "Create Text t with value \"hey\"\n\
         Display Length(t) - 1\n\
         Display t[Length(t) - 1]\n",
        &["2", "'y'"],
    );
}
