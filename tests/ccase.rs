use convert_case::ccase;

#[test]
fn ccase_snake() {
    assert_eq!("my_var_name", ccase!(snake, "my_Var_Name"));
}

#[test]
fn ccase_constant() {
    assert_eq!("MY_VAR_NAME", ccase!(constant, "my_Var_Name"));
}

#[test]
fn ccase_kebab() {
    assert_eq!("my-var-name", ccase!(kebab, "my_Var_Name"));
}

#[test]
fn ccase_kebab_string() {
    assert_eq!("my-var-name", ccase!(kebab, String::from("my_Var_Name")));
}

#[test]
fn ccase_from_kebab_to_camel() {
    assert_eq!("myvarName_var", ccase!(kebab -> camel, "myVar-name_var"));
}

#[test]
fn ccase_from_snake_to_pascal() {
    assert_eq!("My-varName-var", ccase!(snake -> pascal, "my-var_name-var"));
}

#[cfg(feature = "random")]
#[test]
fn ccase_random() {
    assert!((0..10)
        .map(|_| "my-var-name" != ccase!(random, "my_Var_Name"))
        .fold(false, |acc, x| acc || x))
}
