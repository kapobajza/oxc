use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_jest_vi_mocked::{
        TestingFramework, check_assert_type, check_ts_as_expression,
    },
};

#[derive(Debug, Default, Clone)]
pub struct PreferViMocked;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// When working with mocks of functions using Vitest, it's recommended to use the
    /// `vi.mocked()` helper function to properly type the mocked functions. This rule
    /// enforces the use of `vi.mocked()` for better type safety and readability.
    ///
    /// Restricted types:
    /// - `Mock`
    /// - `MockedFunction`
    /// - `MockedClass`
    /// - `MockedObject`
    ///
    /// ### Why is this bad?
    ///
    /// Using type assertions like `fn as Mock` is a less safe approach
    /// than using `vi.mocked()`. The `vi.mocked()` helper provides better
    /// type safety by preserving the original function signature while adding
    /// mock capabilities. It also makes the code more readable and explicit
    /// about mocking intentions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// (foo as vi.Mock).mockReturnValue(1);
    /// const mock = (foo as vi.Mock).mockReturnValue(1);
    /// (foo as unknown as vi.Mock).mockReturnValue(1);
    /// (Obj.foo as vi.Mock).mockReturnValue(1);
    /// ([].foo as vi.Mock).mockReturnValue(1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// vi.mocked(foo).mockReturnValue(1);
    /// const mock = vi.mocked(foo).mockReturnValue(1);
    /// vi.mocked(Obj.foo).mockReturnValue(1);
    /// vi.mocked([].foo).mockReturnValue(1);
    /// ```
    PreferViMocked,
    vitest,
    style,
    conditional_fix,
    version = "next",
);

impl Rule for PreferViMocked {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSAsExpression(ts_expr) = node.kind() {
            if !matches!(ctx.nodes().parent_kind(node.id()), AstKind::TSAsExpression(_)) {
                check_ts_as_expression(node, ts_expr, &TestingFramework::Vitest, ctx);
            }
        } else if let AstKind::TSTypeAssertion(assert_type) = node.kind() {
            check_assert_type(node, assert_type, &TestingFramework::Vitest, ctx);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo();",
        "vi.mocked(foo).mockReturnValue(1);",
        "bar.mockReturnValue(1);",
        "sinon.stub(foo).returns(1);",
        "foo.mockImplementation(() => 1);",
        "obj.foo();",
        "mockFn.mockReturnValue(1);",
        "arr[0]();",
        "obj.foo.mockReturnValue(1);",
        r#"vi.spyOn(obj, "foo").mockReturnValue(1);"#,
        "(foo as Mock.vi).mockReturnValue(1);",
        "type MockType = Mock;
            const mockFn = vi.fn();
            (mockFn as MockType).mockReturnValue(1);",
    ];

    let fail = vec![
        "(foo as Mock).mockReturnValue(1);",
        "(foo as unknown as string as unknown as Mock).mockReturnValue(1);",
        "(foo as unknown as Mock as unknown as Mock).mockReturnValue(1);",
        "(<Mock>foo).mockReturnValue(1);",
        "(foo as Mock).mockImplementation(1);",
        "(foo as unknown as Mock).mockReturnValue(1);",
        "(<Mock>foo as unknown).mockReturnValue(1);",
        "(Obj.foo as Mock).mockReturnValue(1);",
        "([].foo as Mock).mockReturnValue(1);",
        "(foo as MockedFunction).mockReturnValue(1);",
        "(foo as MockedFunction).mockImplementation(1);",
        "(foo as unknown as MockedFunction).mockReturnValue(1);",
        "(Obj.foo as MockedFunction).mockReturnValue(1);",
        "(new Array(0).fill(null).foo as MockedFunction).mockReturnValue(1);",
        "(vi.fn(() => foo) as MockedFunction).mockReturnValue(1);",
        "const mockedUseFocused = useFocused as MockedFunction<typeof useFocused>;",
        "const filter = (MessageService.getMessage as Mock).mock.calls[0][0];",
        "class A {}
            (foo as MockedClass<A>)",
        "(foo as MockedObject<{method: () => void}>)",
        r#"(Obj["foo"] as MockedFunction).mockReturnValue(1);"#,
        "(
            new Array(100)
              .fill(undefined)
              .map(x => x.value)
              .filter(v => !!v).myProperty as MockedFunction<{
              method: () => void;
            }>
            ).mockReturnValue(1);",
    ];

    let fix: Vec<(&str, &str)> = vec![
        ("(foo as Mock).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        (
            "(foo as unknown as string as unknown as Mock).mockReturnValue(1);",
            "(vi.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(foo as unknown as Mock as unknown as Mock).mockReturnValue(1);",
            "(vi.mocked(foo)).mockReturnValue(1);",
        ),
        ("(foo as Mock).mockImplementation(1);", "(vi.mocked(foo)).mockImplementation(1);"),
        ("(foo as unknown as Mock).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        ("(Obj.foo as Mock).mockReturnValue(1);", "(vi.mocked(Obj.foo)).mockReturnValue(1);"),
        ("([].foo as Mock).mockReturnValue(1);", "(vi.mocked([].foo)).mockReturnValue(1);"),
        ("(foo as MockedFunction).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        (
            "(foo as MockedFunction).mockImplementation(1);",
            "(vi.mocked(foo)).mockImplementation(1);",
        ),
        (
            "(foo as unknown as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(Obj.foo as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(Obj.foo)).mockReturnValue(1);",
        ),
        (
            "(new Array(0).fill(null).foo as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(new Array(0).fill(null).foo)).mockReturnValue(1);",
        ),
        (
            "(vi.fn(() => foo) as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(vi.fn(() => foo))).mockReturnValue(1);",
        ),
        (
            "const mockedUseFocused = useFocused as MockedFunction<typeof useFocused>;",
            "const mockedUseFocused = vi.mocked(useFocused);",
        ),
        (
            "const filter = (MessageService.getMessage as Mock).mock.calls[0][0];",
            "const filter = (vi.mocked(MessageService.getMessage)).mock.calls[0][0];",
        ),
        (
            "class A {}
            (foo as MockedClass<A>)",
            "class A {}
            (vi.mocked(foo))",
        ),
        ("(foo as MockedObject<{method: () => void}>)", "(vi.mocked(foo))"),
        (
            r#"(Obj["foo"] as MockedFunction).mockReturnValue(1);"#,
            r#"(vi.mocked(Obj["foo"])).mockReturnValue(1);"#,
        ),
        (
            "(
            new Array(100)
              .fill(undefined)
              .map(x => x.value)
              .filter(v => !!v).myProperty as MockedFunction<{
              method: () => void;
            }>
            ).mockReturnValue(1);",
            "(
            vi.mocked(new Array(100)
              .fill(undefined)
              .map(x => x.value)
              .filter(v => !!v).myProperty)
            ).mockReturnValue(1);",
        ),
    ];

    Tester::new(PreferViMocked::NAME, PreferViMocked::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_plain_ts_file() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail =
        vec!["(<Mock>foo).mockReturnValue(1);", "(<Mock>foo as unknown).mockReturnValue(1);"];

    let fix: Vec<(&str, &str)> = vec![
        ("(<Mock>foo).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        (
            "(<Mock>foo as unknown).mockReturnValue(1);",
            "(vi.mocked(foo) as unknown).mockReturnValue(1);",
        ),
    ];

    Tester::new(PreferViMocked::NAME, PreferViMocked::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("ts")
        .test_and_snapshot();
}
