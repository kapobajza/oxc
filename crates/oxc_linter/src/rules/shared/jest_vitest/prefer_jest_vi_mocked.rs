use std::fmt;

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, TSAsExpression, TSType, TSTypeAssertion, TSTypeName, TSTypeReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{LintContext, ast_util::outermost_paren_parent};

pub enum TestingFramework {
    Jest,
    Vitest,
}

impl fmt::Display for TestingFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestingFramework::Jest => write!(f, "jest"),
            TestingFramework::Vitest => write!(f, "vi"),
        }
    }
}

fn use_jest_vi_mocked(
    span: Span,
    framework: &TestingFramework,
    arg_span: Span,
    ctx: &LintContext<'_>,
) -> OxcDiagnostic {
    let replacement = ctx.source_range(span);
    let arg = ctx.source_range(arg_span);

    OxcDiagnostic::warn(format!("Prefer `{framework}.mocked({arg})` over `{replacement}`."))
        .with_help(format!("Prefer `{framework}.mocked()`"))
        .with_label(span)
}

const MOCK_TYPES: [&str; 4] = ["Mock", "MockedFunction", "MockedClass", "MockedObject"];

pub fn check_ts_as_expression<'a>(
    node: &AstNode<'a>,
    as_expr: &TSAsExpression,
    framework: &TestingFramework,
    ctx: &LintContext<'a>,
) {
    let TSType::TSTypeReference(ts_reference) = &as_expr.type_annotation else {
        return;
    };
    let arg_span = as_expr.expression.get_inner_expression().span();
    check(node, ts_reference, arg_span, as_expr.span, framework, ctx);
}

pub fn check_assert_type<'a>(
    node: &AstNode<'a>,
    assert_type: &TSTypeAssertion,
    framework: &TestingFramework,
    ctx: &LintContext<'a>,
) {
    let TSType::TSTypeReference(ts_reference) = &assert_type.type_annotation else {
        return;
    };
    let arg_span = assert_type.expression.get_inner_expression().span();
    check(node, ts_reference, arg_span, assert_type.span, framework, ctx);
}

fn check<'a>(
    node: &AstNode<'a>,
    ts_reference: &TSTypeReference,
    arg_span: Span,
    span: Span,
    framework: &TestingFramework,
    ctx: &LintContext<'a>,
) {
    match &ts_reference.type_name {
        TSTypeName::QualifiedName(qualified_name) => {
            let TSTypeName::IdentifierReference(ident) = &qualified_name.left else {
                return;
            };

            let framework = match framework {
                TestingFramework::Jest => "jest",
                TestingFramework::Vitest => return,
            };

            if !&ident.name.eq_ignore_ascii_case(framework)
                || !MOCK_TYPES.contains(&qualified_name.right.name.as_str())
            {
                return;
            }
        }
        TSTypeName::IdentifierReference(ident) => {
            if !MOCK_TYPES.contains(&ident.name.as_str()) {
                return;
            }
        }
        TSTypeName::ThisExpression(_) => {
            return;
        }
    }

    if can_fix(node, ctx) {
        ctx.diagnostic_with_fix(use_jest_vi_mocked(span, framework, arg_span, ctx), |fixer| {
            let span_source_code = fixer.source_range(arg_span);
            fixer.replace(span, format!("{framework}.mocked({span_source_code})"))
        });
    } else {
        ctx.diagnostic(use_jest_vi_mocked(span, framework, arg_span, ctx));
    }
}

fn can_fix<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    outermost_paren_parent(node, ctx).is_some_and(|parent| {
        let parent_kind = parent.kind();
        // Disallow fix if parent is AssignmentExpression and node is the left-hand side
        if let AstKind::AssignmentExpression(assign_expr) = parent_kind
            && is_left_hand_side_of_assignment(&assign_expr.left, node)
        {
            return false;
        }
        !matches!(
            parent_kind,
            AstKind::IdentifierReference(_)
                | AstKind::ComputedMemberExpression(_)
                | AstKind::PrivateFieldExpression(_)
        )
    })
}

/// Check if the current node is the left-hand side of an assignment expression
fn is_left_hand_side_of_assignment(assignment_target: &AssignmentTarget, node: &AstNode) -> bool {
    assignment_target.span() == node.span()
}
