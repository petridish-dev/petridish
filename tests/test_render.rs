use petridish::render::Render;
use std::fs;
use tera::Context;

#[test]
fn test_render() {
    let mut context = Context::new();
    context.insert("project", "awesome");
    context.insert("abc", "ABC");
    context.insert("dir_name", "my_dir");
    context.insert("inner", "Bingo");
    context.insert("name", "JoJo");
    context.insert("inner_value", "Secret");

    let output = tempdir::TempDir::new("test").unwrap();
    let render =
        Render::try_new("tests/templates", "{{ project }}", output.path(), context).unwrap();
    render.render().unwrap();

    assert!(output.path().join("awesome").exists());
    assert!(output.path().join("awesome").join("ABC.txt").exists());
    assert_eq!(
        fs::read_to_string(output.path().join("awesome").join("ABC.txt")).unwrap(),
        "JoJo"
    );
    assert!(output
        .path()
        .join("awesome")
        .join("my_dir")
        .join("Bingo.txt")
        .exists());
    assert_eq!(
        fs::read_to_string(
            output
                .path()
                .join("awesome")
                .join("my_dir")
                .join("Bingo.txt")
        )
        .unwrap(),
        "Secret"
    );
}
