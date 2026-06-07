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
pub struct PreferJestMocked;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When working with mocks of functions using Jest, it's recommended to use the
    /// `jest.mocked()` helper function to properly type the mocked functions. This rule
    /// enforces the use of `jest.mocked()` for better type safety and readability.
    ///
    /// Restricted types:
    /// - `jest.Mock`
    /// - `jest.MockedFunction`
    /// - `jest.MockedClass`
    /// - `jest.MockedObject`
    ///
    /// ### Why is this bad?
    ///
    /// Using type assertions like `fn as jest.Mock` is a less safe approach
    /// than using `jest.mocked()`. The `jest.mocked()` helper provides better
    /// type safety by preserving the original function signature while adding
    /// mock capabilities. It also makes the code more readable and explicit
    /// about mocking intentions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// (foo as jest.Mock).mockReturnValue(1);
    /// const mock = (foo as jest.Mock).mockReturnValue(1);
    /// (foo as unknown as jest.Mock).mockReturnValue(1);
    /// (Obj.foo as jest.Mock).mockReturnValue(1);
    /// ([].foo as jest.Mock).mockReturnValue(1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// jest.mocked(foo).mockReturnValue(1);
    /// const mock = jest.mocked(foo).mockReturnValue(1);
    /// jest.mocked(Obj.foo).mockReturnValue(1);
    /// jest.mocked([].foo).mockReturnValue(1);
    /// ```
    PreferJestMocked,
    jest,
    style,
    conditional_fix,
    version = "0.5.0",
);

impl Rule for PreferJestMocked {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSAsExpression(ts_expr) = node.kind() {
            if !matches!(ctx.nodes().parent_kind(node.id()), AstKind::TSAsExpression(_)) {
                check_ts_as_expression(node, ts_expr, &TestingFramework::Jest, ctx);
            }
        } else if let AstKind::TSTypeAssertion(assert_type) = node.kind() {
            check_assert_type(node, assert_type, &TestingFramework::Jest, ctx);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo();", None, None, None),
        ("jest.mocked(foo).mockReturnValue(1);", None, None, None),
        ("bar.mockReturnValue(1);", None, None, None),
        ("sinon.stub(foo).returns(1);", None, None, None),
        ("foo.mockImplementation(() => 1);", None, None, None),
        ("obj.foo();", None, None, None),
        ("mockFn.mockReturnValue(1);", None, None, None),
        ("arr[0]();", None, None, None),
        ("obj.foo.mockReturnValue(1);", None, None, None),
        ("jest.spyOn(obj, 'foo').mockReturnValue(1);", None, None, None),
        ("(foo as Mock.jest).mockReturnValue(1);", None, None, None),
        (
            "
                type MockType = jest.Mock;
                const mockFn = jest.fn();
                (mockFn as MockType).mockReturnValue(1);
            ",
            None,
            None,
            None,
        ),
    ];

    let fail = vec![
        ("(foo as jest.Mock).mockReturnValue(1);", None, None, None),
        (
            "(foo as unknown as string as unknown as jest.Mock).mockReturnValue(1);",
            None,
            None,
            None,
        ),
        (
            "(foo as unknown as jest.Mock as unknown as jest.Mock).mockReturnValue(1);",
            None,
            None,
            None,
        ),
        ("(foo as jest.Mock).mockImplementation(1);", None, None, None),
        ("(foo as unknown as jest.Mock).mockReturnValue(1);", None, None, None),
        ("(Obj.foo as jest.Mock).mockReturnValue(1);", None, None, None),
        ("([].foo as jest.Mock).mockReturnValue(1);", None, None, None),
        ("(foo as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        ("(foo as jest.MockedFunction).mockImplementation(1);", None, None, None),
        ("(foo as unknown as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        ("(Obj.foo as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        (
            "(new Array(0).fill(null).foo as jest.MockedFunction).mockReturnValue(1);",
            None,
            None,
            None,
        ),
        ("(jest.fn(() => foo) as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        (
            "const mockedUseFocused = useFocused as jest.MockedFunction<typeof useFocused>;",
            None,
            None,
            None,
        ),
        (
            "const filter = (MessageService.getMessage as jest.Mock).mock.calls[0][0];",
            None,
            None,
            None,
        ),
        (
            "
                class A {}
                (foo as jest.MockedClass<A>)
            ",
            None,
            None,
            None,
        ),
        ("(foo as jest.MockedObject<{method: () => void}>)", None, None, None),
        ("(Obj['foo'] as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        (
            "
                (
                new Array(100)
                    .fill(undefined)
                    .map(x => x.value)
                    .filter(v => !!v).myProperty as jest.MockedFunction<{
                    method: () => void;
                }>
                ).mockReturnValue(1);
            ",
            None,
            None,
            None,
        ),
    ];

    let fix = vec![
        ("(foo as jest.Mock).mockReturnValue(1);", "(jest.mocked(foo)).mockReturnValue(1);"),
        (
            "(foo as unknown as string as unknown as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(foo as unknown as jest.Mock as unknown as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        ("(foo as jest.Mock).mockImplementation(1);", "(jest.mocked(foo)).mockImplementation(1);"),
        (
            "(foo as unknown as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(Obj.foo as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(Obj.foo)).mockReturnValue(1);",
        ),
        ("([].foo as jest.Mock).mockReturnValue(1);", "(jest.mocked([].foo)).mockReturnValue(1);"),
        (
            "(foo as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(foo as jest.MockedFunction).mockImplementation(1);",
            "(jest.mocked(foo)).mockImplementation(1);",
        ),
        (
            "(foo as unknown as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(Obj.foo as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(Obj.foo)).mockReturnValue(1);",
        ),
        (
            "(new Array(0).fill(null).foo as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(new Array(0).fill(null).foo)).mockReturnValue(1);",
        ),
        (
            "(jest.fn(() => foo) as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(jest.fn(() => foo))).mockReturnValue(1);",
        ),
        (
            "const mockedUseFocused = useFocused as jest.MockedFunction<typeof useFocused>;",
            "const mockedUseFocused = jest.mocked(useFocused);",
        ),
        (
            "const filter = (MessageService.getMessage as jest.Mock).mock.calls[0][0];",
            "const filter = (jest.mocked(MessageService.getMessage)).mock.calls[0][0];",
        ),
        (
            "
                class A {}
                (foo as jest.MockedClass<A>)
            ",
            "
                class A {}
                (jest.mocked(foo))
            ",
        ),
        ("(foo as jest.MockedObject<{method: () => void}>)", "(jest.mocked(foo))"),
        (
            "(Obj['foo'] as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(Obj['foo'])).mockReturnValue(1);",
        ),
        (
            "
                (
                new Array(100)
                    .fill(undefined)
                    .map(x => x.value)
                    .filter(v => !!v).myProperty as jest.MockedFunction<{
                    method: () => void;
                }>
                ).mockReturnValue(1);
            ",
            "
                (
                jest.mocked(new Array(100)
                    .fill(undefined)
                    .map(x => x.value)
                    .filter(v => !!v).myProperty)
                ).mockReturnValue(1);
            ",
        ),
        // we can't fix this case, as fixing it would result in a syntax error
        // (we'd be attempting attempting to assign `jest.fn()` to invalid left-hand side)
        ("(foo as jest.Mock) = jest.fn();", "(foo as jest.Mock) = jest.fn();"),
    ];

    Tester::new(PreferJestMocked::NAME, PreferJestMocked::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_plain_ts_file() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![
        "(<jest.Mock>foo).mockReturnValue(1);",
        "(<jest.Mock>foo as unknown).mockReturnValue(1);",
    ];

    let fix: Vec<(&str, &str)> = vec![
        ("(<jest.Mock>foo).mockReturnValue(1);", "(jest.mocked(foo)).mockReturnValue(1);"),
        (
            "(<jest.Mock>foo as unknown).mockReturnValue(1);",
            "(jest.mocked(foo) as unknown).mockReturnValue(1);",
        ),
    ];

    Tester::new(PreferJestMocked::NAME, PreferJestMocked::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("ts")
        .test_and_snapshot();
}
