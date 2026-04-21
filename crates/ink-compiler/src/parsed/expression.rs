use std::fmt;

use super::{Divert, Identifier, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone, PartialEq)]
pub enum NumberValue {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Number(NumberValue),
    StringExpression,
    VariableReference {
        path: Vec<Identifier>,
        is_constant_reference: bool,
        is_list_item_reference: bool,
    },
    FunctionCall {
        function_name: Identifier,
        should_pop_returned_value: bool,
    },
    DivertTarget,
    List {
        item_identifiers: Vec<Identifier>,
    },
    Binary {
        op_name: String,
    },
    Unary {
        op: String,
    },
    IncDec {
        identifier: Identifier,
        is_inc: bool,
    },
    MultipleCondition,
}

fn new_expression_object(kind: ExpressionKind) -> ObjectRef {
    let object = Object::new_ref();
    object.borrow_mut().set_expression_kind(kind);
    object
}

#[derive(Debug, Clone)]
pub struct Number {
    object: ObjectRef,
}

impl Number {
    pub fn new(value: NumberValue) -> Self {
        Self {
            object: new_expression_object(ExpressionKind::Number(value)),
        }
    }

    pub fn int(value: i64) -> Self {
        Self::new(NumberValue::Int(value))
    }

    pub fn float(value: f64) -> Self {
        Self::new(NumberValue::Float(value))
    }

    pub fn bool(value: bool) -> Self {
        Self::new(NumberValue::Bool(value))
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn value(&self) -> Option<NumberValue> {
        match self.object.borrow().kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::Number(value),
            } => Some(value.clone()),
            _ => None,
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value() {
            Some(value) => write!(f, "{value}"),
            None => f.write_str("<invalid number>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringExpression {
    object: ObjectRef,
}

impl StringExpression {
    pub fn new(content: Vec<ObjectRef>) -> Self {
        let object = new_expression_object(ExpressionKind::StringExpression);
        for child in content {
            Object::add_content(&object, child);
        }

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }

    pub fn is_single_string(&self) -> bool {
        if self.content().len() != 1 {
            return false;
        }

        matches!(self.content()[0].borrow().kind(), ObjectKind::Text { .. })
    }

    pub fn text(&self) -> String {
        self.content()
            .iter()
            .filter_map(|child| match child.borrow().kind() {
                ObjectKind::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }
}

impl fmt::Display for StringExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text())
    }
}

#[derive(Debug, Clone)]
pub struct VariableReference {
    object: ObjectRef,
}

impl VariableReference {
    pub fn new(path: Vec<Identifier>) -> Self {
        Self {
            object: new_expression_object(ExpressionKind::VariableReference {
                path,
                is_constant_reference: false,
                is_list_item_reference: false,
            }),
        }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn path(&self) -> Vec<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::VariableReference { path, .. },
            } => path.clone(),
            _ => Vec::new(),
        }
    }

    pub fn name(&self) -> String {
        self.path()
            .into_iter()
            .map(|identifier| identifier.name)
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl fmt::Display for VariableReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    object: ObjectRef,
}

impl FunctionCall {
    pub fn new(function_name: Identifier, arguments: Vec<ObjectRef>) -> Self {
        let object = new_expression_object(ExpressionKind::FunctionCall {
            function_name,
            should_pop_returned_value: false,
        });

        for argument in arguments {
            Object::add_content(&object, argument);
        }

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn name(&self) -> Option<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::FunctionCall { function_name, .. },
            } => Some(function_name.clone()),
            _ => None,
        }
    }

    pub fn arguments(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .name()
            .map(|identifier| identifier.name)
            .unwrap_or_else(|| "<invalid function>".to_string());
        write!(f, "{name}()")
    }
}

#[derive(Debug, Clone)]
pub struct DivertTarget {
    object: ObjectRef,
}

impl DivertTarget {
    pub fn new(divert: Divert) -> Self {
        let object = new_expression_object(ExpressionKind::DivertTarget);
        Object::add_content(&object, divert.object());

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn divert(&self) -> Option<Divert> {
        self.object
            .borrow()
            .content()
            .first()
            .cloned()
            .map(Divert::from_object)
    }
}

impl fmt::Display for DivertTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.divert() {
            Some(divert) => write!(f, "{divert}"),
            None => f.write_str("->"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct List {
    object: ObjectRef,
}

impl List {
    pub fn new(item_identifiers: Vec<Identifier>) -> Self {
        Self {
            object: new_expression_object(ExpressionKind::List { item_identifiers }),
        }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn item_identifiers(&self) -> Vec<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::List { item_identifiers },
            } => item_identifiers.clone(),
            _ => Vec::new(),
        }
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self
            .item_identifiers()
            .into_iter()
            .map(|identifier| identifier.name)
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "({items})")
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    object: ObjectRef,
}

impl BinaryExpression {
    pub fn new(left: ObjectRef, right: ObjectRef, op_name: impl Into<String>) -> Self {
        let object = new_expression_object(ExpressionKind::Binary {
            op_name: op_name.into(),
        });
        Object::add_content(&object, left);
        Object::add_content(&object, right);
        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn op_name(&self) -> Option<String> {
        match self.object.borrow().kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::Binary { op_name },
            } => Some(op_name.clone()),
            _ => None,
        }
    }

    pub fn left_expression(&self) -> Option<ObjectRef> {
        self.object.borrow().content().first().cloned()
    }

    pub fn right_expression(&self) -> Option<ObjectRef> {
        self.object.borrow().content().get(1).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    object: ObjectRef,
}

impl UnaryExpression {
    pub fn new(inner: ObjectRef, op: impl Into<String>) -> Self {
        let object = new_expression_object(ExpressionKind::Unary { op: op.into() });
        Object::add_content(&object, inner);
        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn op(&self) -> Option<String> {
        match self.object.borrow().kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::Unary { op },
            } => Some(op.clone()),
            _ => None,
        }
    }

    pub fn inner_expression(&self) -> Option<ObjectRef> {
        self.object.borrow().content().first().cloned()
    }
}

#[derive(Debug, Clone)]
pub struct IncDecExpression {
    object: ObjectRef,
}

impl IncDecExpression {
    pub fn new(identifier: Identifier, is_inc: bool, expression: Option<ObjectRef>) -> Self {
        let object = new_expression_object(ExpressionKind::IncDec { identifier, is_inc });
        if let Some(expression) = expression {
            Object::add_content(&object, expression);
        }
        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }
}

#[derive(Debug, Clone)]
pub struct MultipleConditionExpression {
    object: ObjectRef,
}

impl MultipleConditionExpression {
    pub fn new(condition_expressions: Vec<ObjectRef>) -> Self {
        let object = new_expression_object(ExpressionKind::MultipleCondition);
        for expression in condition_expressions {
            Object::add_content(&object, expression);
        }
        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn sub_expressions(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsed::Text;

    #[test]
    fn expression_wrappers_store_metadata_and_content() {
        let number = Number::int(42);
        assert_eq!(number.value(), Some(NumberValue::Int(42)));

        let variable = VariableReference::new(vec![Identifier::new("foo"), Identifier::new("bar")]);
        assert_eq!(variable.name(), "foo.bar");

        let string = StringExpression::new(vec![Text::new("hello").object()]);
        assert!(string.is_single_string());
        assert_eq!(string.text(), "hello");

        let list = List::new(vec![Identifier::new("a"), Identifier::new("b")]);
        assert_eq!(list.item_identifiers().len(), 2);

        let binary = BinaryExpression::new(Number::int(1).object(), Number::int(2).object(), "+");
        assert_eq!(binary.op_name().as_deref(), Some("+"));
        assert_eq!(binary.left_expression().is_some(), true);
        assert_eq!(binary.right_expression().is_some(), true);

        let unary = UnaryExpression::new(Number::bool(true).object(), "not");
        assert_eq!(unary.op().as_deref(), Some("not"));

        let multiple = MultipleConditionExpression::new(vec![
            Number::bool(true).object(),
            Number::bool(false).object(),
        ]);
        assert_eq!(multiple.sub_expressions().len(), 2);
    }
}
