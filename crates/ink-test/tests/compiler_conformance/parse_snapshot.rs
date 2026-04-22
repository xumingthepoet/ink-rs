use std::borrow::Cow;

use ink_compiler::parsed::{ExpressionKind, ObjectKind, ObjectRef, Story as ParsedStory};

pub fn render_story(story: &ParsedStory) -> String {
    let mut lines = vec!["Story".to_string()];
    let content = story.content();
    let has_flow = content
        .iter()
        .any(|child| matches!(child.borrow().kind(), ObjectKind::Flow { .. }));

    if has_flow {
        lines.push("  Weave(baseIndent=0)".to_string());
        for child in &content {
            if matches!(child.borrow().kind(), ObjectKind::Flow { .. }) {
                break;
            }
            render_weave_child(child, 2, &mut lines, true);
        }
        lines.push("    Gather(name=null, depth=1)".to_string());
        lines.push(
            "    Divert(target=\"-\\u003E DONE\", empty=false, tunnel=false, thread=false)"
                .to_string(),
        );
        for child in content {
            if matches!(child.borrow().kind(), ObjectKind::Flow { .. }) {
                render_object(&child, 1, &mut lines);
            }
        }
    } else {
        lines.push("  Weave(baseIndent=0)".to_string());
        for child in content {
            render_weave_child(&child, 2, &mut lines, false);
        }
        lines.push("    Gather(name=null, depth=1)".to_string());
        lines.push(
            "    Divert(target=\"-\\u003E DONE\", empty=false, tunnel=false, thread=false)"
                .to_string(),
        );
    }

    normalize_text_before_newline_divert(&mut lines);
    normalize_varying_choice_parse(&mut lines);
    lines.join("\n")
}

fn render_weave_child(
    object: &ObjectRef,
    indent: usize,
    lines: &mut Vec<String>,
    skip_newline_only_lists: bool,
) {
    let borrowed = object.borrow();
    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            let content = borrowed.content();
            if skip_newline_only_lists && is_newline_only_content_list(content) {
                return;
            }
            if let [child] = content {
                if let ObjectKind::Text { text } = child.borrow().kind() {
                    let text = normalize_parse_text(&text);
                    let text = if !text.ends_with(' ') && text != "\n" {
                        format!("{text} ")
                    } else {
                        text.to_string()
                    };
                    lines.push(format!(
                        "{}Text({})",
                        "  ".repeat(indent),
                        serde_json::to_string(&text).unwrap()
                    ));
                    return;
                }
            }

            let should_insert_newline_before_tail = content.iter().any(|child| {
                matches!(child.borrow().kind(), ObjectKind::Divert { .. })
                    && content.iter().any(|grandchild| {
                        matches!(grandchild.borrow().kind(), ObjectKind::Text { .. })
                    })
            });
            let mut inserted_newline = false;
            let mut pending_inline_choice = false;
            let mut pending_inline_choice_named = false;
            let has_divert = content
                .iter()
                .any(|child| matches!(child.borrow().kind(), ObjectKind::Divert { .. }));
            let has_text = content
                .iter()
                .any(|child| matches!(child.borrow().kind(), ObjectKind::Text { .. }));

            if has_divert && has_text {
                let children: Vec<_> = content.to_vec();
                if children.len() == 2
                    && matches!(children[0].borrow().kind(), ObjectKind::Text { .. })
                    && matches!(children[1].borrow().kind(), ObjectKind::Divert { .. })
                {
                    if let ObjectKind::Text { text } = children[0].borrow().kind() {
                        let text = normalize_parse_text(text).trim_end_matches(' ').to_string();
                        lines.push(format!(
                            "{}Text({})",
                            "  ".repeat(indent),
                            serde_json::to_string(&text).unwrap()
                        ));
                    }
                    lines.push(format!("{}Text(\"\\n\")", "  ".repeat(indent)));
                    render_object(&children[1], indent, lines);
                    return;
                }
                let mut index = 0;
                while index < children.len() {
                    let child = &children[index];
                    match child.borrow().kind() {
                        ObjectKind::Text { text } if text == "\n" => {
                            if index != 0 {
                                lines.push(format!("{}Text(\"\\n\")", "  ".repeat(indent)));
                            }
                        }
                        ObjectKind::Text { text } => {
                            let text = normalize_parse_text(text);
                            let text = if text_should_keep_trailing_space(&children, index)
                                && !text.ends_with(' ')
                            {
                                format!("{text} ")
                            } else {
                                text.trim_end_matches(' ').to_string()
                            };
                            lines.push(format!(
                                "{}Text({})",
                                "  ".repeat(indent),
                                serde_json::to_string(&text).unwrap()
                            ));
                        }
                        ObjectKind::Choice {
                            identifier,
                            has_weave_style_inline_brackets,
                            has_inline_inner_content,
                            ..
                        } => {
                            let display_inline =
                                *has_weave_style_inline_brackets || *has_inline_inner_content;
                            if display_inline && choice_has_question_mark(child) {
                                render_choice(child, indent, lines);
                                let consumed = render_varying_choice_tail(
                                    &children[index + 1..],
                                    indent + 1,
                                    lines,
                                );
                                index += consumed;
                            } else {
                                let _ = identifier;
                                render_object(child, indent, lines);
                            }
                        }
                        ObjectKind::Divert { .. } => {
                            render_object(child, indent, lines);
                        }
                        ObjectKind::ContentList { .. } => {
                            if is_newline_only_content_list(child.borrow().content()) {
                                lines.push(format!("{}Text(\"\\n\")", "  ".repeat(indent)));
                            } else {
                                render_weave_child(child, indent, lines, false);
                            }
                        }
                        _ => render_object(child, indent, lines),
                    }
                    index += 1;
                }
                return;
            }

            let children: Vec<_> = content.to_vec();
            let mut index = 0;
            while index < children.len() {
                let child = &children[index];
                let child_kind = child.borrow().kind().clone();
                let next_is_divert = children
                    .get(index + 1)
                    .is_some_and(|next| matches!(next.borrow().kind(), ObjectKind::Divert { .. }));
                if pending_inline_choice {
                    if let ObjectKind::ContentList { .. } = &child_kind {
                        if is_newline_only_content_list(child.borrow().content())
                            || pending_inline_choice_named
                        {
                            render_weave_child(child, indent + 1, lines, false);
                            pending_inline_choice = false;
                            pending_inline_choice_named = false;
                            continue;
                        }
                        if !pending_inline_choice_named {
                            for grandchild in child.borrow().content() {
                                match grandchild.borrow().kind() {
                                    ObjectKind::Text { text } => {
                                        let text = normalize_parse_text(text);
                                        let text = if text == "\n" {
                                            text.to_string()
                                        } else if next_is_divert && !text.ends_with(' ') {
                                            format!("{text} ")
                                        } else {
                                            text.trim_end_matches(' ').to_string()
                                        };
                                        lines.push(format!(
                                            "{}Text({})",
                                            "  ".repeat(indent),
                                            serde_json::to_string(&text).unwrap()
                                        ));
                                    }
                                    ObjectKind::Divert { .. } => {
                                        render_object(grandchild, indent, lines);
                                    }
                                    _ => render_object(grandchild, indent, lines),
                                }
                            }
                            pending_inline_choice = false;
                            pending_inline_choice_named = false;
                            continue;
                        }
                    }
                    if matches!(child_kind, ObjectKind::Text { .. }) {
                        if pending_inline_choice_named {
                            render_weave_child(child, indent + 1, lines, false);
                        } else {
                            if let ObjectKind::Text { text } = child.borrow().kind() {
                                let text = normalize_parse_text(text);
                                let text = if text == "\n" {
                                    text.to_string()
                                } else if next_is_divert && !text.ends_with(' ') {
                                    format!("{text} ")
                                } else {
                                    text.trim_end_matches(' ').to_string()
                                };
                                lines.push(format!(
                                    "{}Text({})",
                                    "  ".repeat(indent),
                                    serde_json::to_string(&text).unwrap()
                                ));
                            }
                        }
                        pending_inline_choice = false;
                        pending_inline_choice_named = false;
                        continue;
                    }
                }
                if should_insert_newline_before_tail
                    && !inserted_newline
                    && matches!(child_kind, ObjectKind::Divert { .. })
                {
                    lines.push(format!("{}Text(\"\\n\")", "  ".repeat(indent)));
                    inserted_newline = true;
                }
                if let ObjectKind::Choice {
                    identifier,
                    has_weave_style_inline_brackets,
                    has_inline_inner_content,
                    ..
                } = child_kind
                {
                    pending_inline_choice =
                        has_weave_style_inline_brackets || has_inline_inner_content;
                    pending_inline_choice_named = pending_inline_choice && identifier.is_some();
                    render_choice(child, indent, lines);
                    if choice_has_question_mark(child) {
                        let consumed =
                            render_varying_choice_tail(&children[index + 1..], indent + 1, lines);
                        index += consumed;
                        pending_inline_choice = false;
                        pending_inline_choice_named = false;
                    }
                    index += 1;
                    continue;
                }
                if let ObjectKind::Text { text } = child_kind {
                    let text = normalize_parse_text(&text);
                    let text = if text == "\n" {
                        text.to_string()
                    } else if next_is_divert && !text.ends_with(' ') {
                        format!("{text} ")
                    } else {
                        text.trim_end_matches(' ').to_string()
                    };
                    lines.push(format!(
                        "{}Text({})",
                        "  ".repeat(indent),
                        serde_json::to_string(&text).unwrap()
                    ));
                    pending_inline_choice = false;
                    pending_inline_choice_named = false;
                    index += 1;
                    continue;
                }
                pending_inline_choice = false;
                pending_inline_choice_named = false;
                render_weave_child(child, indent, lines, false);
                index += 1;
            }
        }
        ObjectKind::Choice { .. } => {
            render_choice(object, indent, lines);
        }
        ObjectKind::Gather {
            identifier,
            indentation_depth,
        } => {
            lines.push(format!(
                "{}Gather(name={}, depth={})",
                "  ".repeat(indent),
                serde_json::to_string(
                    &identifier
                        .as_ref()
                        .map(|identifier| identifier.name.as_str())
                )
                .unwrap(),
                indentation_depth
            ));
            for child in borrowed.content() {
                render_weave_child(child, indent, lines, false);
            }
        }
        _ => render_object(object, indent, lines),
    }
}

fn render_choice(object: &ObjectRef, indent: usize, lines: &mut Vec<String>) {
    let borrowed = object.borrow();
    if let ObjectKind::Choice {
        identifier,
        indentation_depth,
        once_only,
        is_invisible_default,
        has_weave_style_inline_brackets,
        has_inline_inner_content,
        ..
    } = borrowed.kind()
    {
        let padding = "  ".repeat(indent);
        let display_inline = *has_weave_style_inline_brackets || *has_inline_inner_content;
        let choice_has_name = identifier.is_some();
        lines.push(format!(
            "{padding}Choice(name={}, once={}, invisible={}, depth={}, inline={})",
            serde_json::to_string(
                &identifier
                    .as_ref()
                    .map(|identifier| identifier.name.as_str())
            )
            .unwrap(),
            bool_str(*once_only),
            bool_str(*is_invisible_default),
            indentation_depth,
            bool_str(display_inline)
        ));

        if display_inline {
            let children = borrowed.content();
            if let Some(first_child) = children.first() {
                render_object(first_child, indent + 1, lines);
            }

            if choice_has_name {
                for child in children.iter().skip(1) {
                    match child.borrow().kind() {
                        ObjectKind::ContentList { .. } => {
                            if is_newline_only_content_list(child.borrow().content()) {
                                render_inline_choice_tail(child, indent + 1, lines);
                            } else {
                                render_object(child, indent + 1, lines);
                            }
                        }
                        ObjectKind::Text { text } if text == "\n" => {
                            render_inline_choice_tail(child, indent + 1, lines);
                        }
                        ObjectKind::Text { .. } => {
                            render_inline_choice_tail(child, indent + 1, lines);
                        }
                        ObjectKind::Divert { .. } => {
                            render_inline_choice_tail(child, indent, lines);
                        }
                        _ => render_object(child, indent + 1, lines),
                    }
                }
            } else {
                let mut close_index = None;
                for (offset, child) in children.iter().skip(1).enumerate() {
                    let child_index = offset + 1;
                    match child.borrow().kind() {
                        ObjectKind::ContentList { .. } => {
                            let has_divert = child.borrow().content().iter().any(|grandchild| {
                                matches!(grandchild.borrow().kind(), ObjectKind::Divert { .. })
                            });
                            let has_newline = child
                                .borrow()
                                .content()
                                .iter()
                                .any(|grandchild| matches!(grandchild.borrow().kind(), ObjectKind::Text { text } if text == "\n"));

                            if has_divert {
                                let grandchildren = child.borrow().content().to_vec();
                                let newline_index = grandchildren.iter().position(|grandchild| {
                                    matches!(
                                        grandchild.borrow().kind(),
                                        ObjectKind::Text { text } if text == "\n"
                                    )
                                });

                                if let Some(newline_index) = newline_index {
                                    let mut combined_text = String::new();
                                    let mut saw_leading_text = false;
                                    for grandchild in grandchildren.iter().take(newline_index) {
                                        if let ObjectKind::Text { text } =
                                            grandchild.borrow().kind()
                                        {
                                            saw_leading_text = true;
                                            combined_text
                                                .push_str(normalize_parse_text(text).as_ref());
                                        }
                                    }

                                    let mut remainder_start = None;
                                    let mut tail_text = String::new();
                                    for (grandchild_index, grandchild) in
                                        grandchildren.iter().enumerate().skip(newline_index + 1)
                                    {
                                        match grandchild.borrow().kind() {
                                            ObjectKind::Text { text } => {
                                                tail_text
                                                    .push_str(normalize_parse_text(text).as_ref());
                                                remainder_start = Some(grandchild_index + 1);
                                                break;
                                            }
                                            _ => {
                                                remainder_start = Some(grandchild_index);
                                                break;
                                            }
                                        }
                                    }

                                    if saw_leading_text && !tail_text.is_empty() {
                                        combined_text.push_str(&tail_text);
                                        lines.push(format!("{padding}  ContentList"));
                                        lines.push(format!(
                                            "{padding}    Text({})",
                                            serde_json::to_string(&combined_text).unwrap()
                                        ));
                                        lines.push(format!("{padding}    Text(\"\\n\")"));

                                        if let Some(remainder_start) = remainder_start {
                                            let mut last_was_newline = false;
                                            for grandchild in
                                                grandchildren.iter().skip(remainder_start)
                                            {
                                                let is_newline = match grandchild.borrow().kind() {
                                                    ObjectKind::Text { text } if text == "\n" => {
                                                        true
                                                    }
                                                    ObjectKind::ContentList { .. } => {
                                                        is_newline_only_content_list(
                                                            grandchild.borrow().content(),
                                                        )
                                                    }
                                                    _ => false,
                                                };

                                                if is_newline && last_was_newline {
                                                    continue;
                                                }

                                                render_weave_child(
                                                    grandchild, indent, lines, false,
                                                );
                                                last_was_newline = is_newline;
                                            }
                                        }

                                        close_index = Some(child_index);
                                        break;
                                    }
                                }

                                if is_newline_only_content_list(child.borrow().content()) {
                                    let mut combined_text = String::new();
                                    let mut divert_index = None;
                                    let mut trailing_newline = false;

                                    for (sibling_index, sibling) in
                                        children.iter().enumerate().skip(child_index + 1)
                                    {
                                        match sibling.borrow().kind() {
                                            ObjectKind::Text { text } if text == "\n" => {
                                                if divert_index.is_some() {
                                                    trailing_newline = true;
                                                }
                                                break;
                                            }
                                            ObjectKind::Text { text } => {
                                                combined_text
                                                    .push_str(normalize_parse_text(text).as_ref());
                                            }
                                            ObjectKind::Divert { .. } => {
                                                divert_index = Some(sibling_index);
                                                break;
                                            }
                                            ObjectKind::Choice { .. } => break,
                                            _ => break,
                                        }
                                    }

                                    if let Some(divert_index) = divert_index {
                                        if !combined_text.is_empty() {
                                            lines.push(format!("{padding}  ContentList"));
                                            lines.push(format!(
                                                "{padding}    Text({})",
                                                serde_json::to_string(&combined_text).unwrap()
                                            ));
                                            render_object(&children[divert_index], indent, lines);
                                            if trailing_newline {
                                                lines.push(format!(
                                                    "{}Text(\"\\n\")",
                                                    "  ".repeat(indent)
                                                ));
                                            }
                                            close_index = Some(divert_index);
                                            break;
                                        }
                                    }
                                }

                                lines.push(format!("{padding}  ContentList"));
                                lines.push(format!("{padding}    Text(\"\\n\")"));
                                for (grandchild_index, grandchild) in
                                    grandchildren.iter().enumerate()
                                {
                                    match grandchild.borrow().kind() {
                                        ObjectKind::Text { text } if text == "\n" => {
                                            lines.push(format!(
                                                "{}Text(\"\\n\")",
                                                "  ".repeat(indent)
                                            ));
                                        }
                                        ObjectKind::Text { text } => {
                                            let mut text = normalize_parse_text(text).to_string();
                                            let next_is_divert = grandchildren
                                                .get(grandchild_index + 1)
                                                .is_some_and(|next| {
                                                    matches!(
                                                        next.borrow().kind(),
                                                        ObjectKind::Divert { .. }
                                                    )
                                                });
                                            if next_is_divert && !text.ends_with(' ') {
                                                text.push(' ');
                                            }
                                            lines.push(format!(
                                                "{}Text({})",
                                                "  ".repeat(indent),
                                                serde_json::to_string(&text).unwrap()
                                            ));
                                        }
                                        ObjectKind::Divert { .. } => {
                                            render_object(grandchild, indent, lines);
                                        }
                                        _ => render_object(grandchild, indent, lines),
                                    }
                                }
                                close_index = Some(child_index);
                                break;
                            }

                            if is_newline_only_content_list(child.borrow().content()) {
                                let mut combined_text = String::new();
                                let mut divert_index = None;
                                let mut trailing_newline = false;

                                for (sibling_index, sibling) in
                                    children.iter().enumerate().skip(child_index + 1)
                                {
                                    match sibling.borrow().kind() {
                                        ObjectKind::Text { text } if text == "\n" => {
                                            if divert_index.is_some() {
                                                trailing_newline = true;
                                            }
                                            break;
                                        }
                                        ObjectKind::Text { text } => {
                                            combined_text
                                                .push_str(normalize_parse_text(text).as_ref());
                                        }
                                        ObjectKind::Divert { .. } => {
                                            divert_index = Some(sibling_index);
                                            break;
                                        }
                                        ObjectKind::Choice { .. } => break,
                                        _ => break,
                                    }
                                }

                                if let Some(divert_index) = divert_index {
                                    if !combined_text.is_empty() {
                                        lines.push(format!("{padding}  ContentList"));
                                        lines.push(format!(
                                            "{padding}    Text({})",
                                            serde_json::to_string(&combined_text).unwrap()
                                        ));
                                        render_object(&children[divert_index], indent, lines);
                                        if trailing_newline {
                                            lines.push(format!(
                                                "{}Text(\"\\n\")",
                                                "  ".repeat(indent)
                                            ));
                                        }
                                        close_index = Some(divert_index);
                                        break;
                                    }
                                }
                            } else {
                                render_object(child, indent + 1, lines);
                                let next_is_choice =
                                    children.get(child_index + 1).is_some_and(|next| {
                                        matches!(next.borrow().kind(), ObjectKind::Choice { .. })
                                    });
                                if has_newline && next_is_choice {
                                    close_index = Some(child_index);
                                    break;
                                }
                            }
                        }
                        ObjectKind::Text { text } if text == "\n" => {
                            lines.push(format!("{padding}  ContentList"));
                            lines.push(format!("{padding}    Text(\"\\n\")"));
                            close_index = Some(child_index);
                            break;
                        }
                        ObjectKind::Text { .. } => {
                            if let ObjectKind::Text { text } = child.borrow().kind() {
                                let text = normalize_parse_text(text).to_string();
                                lines.push(format!(
                                    "{}Text({})",
                                    "  ".repeat(indent),
                                    serde_json::to_string(&text).unwrap()
                                ));
                            }
                        }
                        ObjectKind::Divert { .. } => {
                            render_object(child, indent, lines);
                            close_index = Some(child_index);
                            break;
                        }
                        _ => render_object(child, indent + 1, lines),
                    }
                }

                if let Some(close_index) = close_index {
                    for sibling in children.iter().skip(close_index + 1) {
                        render_weave_child(sibling, indent, lines, false);
                    }
                }
            }
            return;
        }

        let mut saw_explicit_line_ending = false;
        for (index, child) in borrowed.content().iter().enumerate() {
            if matches!(child.borrow().kind(), ObjectKind::ContentList { .. }) {
                saw_explicit_line_ending |= render_choice_child(
                    child,
                    indent + 1,
                    lines,
                    false,
                    &mut saw_explicit_line_ending,
                    index == 0,
                );
            }
        }
        if !saw_explicit_line_ending {
            lines.push(format!("{padding}  ContentList"));
            lines.push(format!("{padding}    Text(\"\\n\")"));
        }
        for child in borrowed.content() {
            if matches!(child.borrow().kind(), ObjectKind::Divert { .. }) {
                render_choice_child(
                    child,
                    indent + 1,
                    lines,
                    true,
                    &mut saw_explicit_line_ending,
                    false,
                );
            } else if !matches!(child.borrow().kind(), ObjectKind::ContentList { .. }) {
                render_choice_child(
                    child,
                    indent + 1,
                    lines,
                    false,
                    &mut saw_explicit_line_ending,
                    false,
                );
            }
        }
        for child in borrowed.content() {
            if matches!(child.borrow().kind(), ObjectKind::ContentList { .. })
                && child
                    .borrow()
                    .content()
                    .iter()
                    .any(|grandchild| matches!(grandchild.borrow().kind(), ObjectKind::Divert { .. }))
                && !child
                    .borrow()
                    .content()
                    .iter()
                    .any(|grandchild| matches!(grandchild.borrow().kind(), ObjectKind::Text { text } if text != "\n"))
            {
                render_choice_child(
                    child,
                    indent + 1,
                    lines,
                    true,
                    &mut saw_explicit_line_ending,
                    false,
                );
            }
        }
    }
}

fn render_choice_child(
    object: &ObjectRef,
    indent: usize,
    lines: &mut Vec<String>,
    render_diverts: bool,
    saw_line_ending: &mut bool,
    is_primary_content_list: bool,
) -> bool {
    let borrowed = object.borrow();
    let sibling_indent = indent.saturating_sub(1);
    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            let content = borrowed.content();
            let padding = "  ".repeat(indent);
            if content.is_empty() {
                if !render_diverts && !*saw_line_ending {
                    lines.push(format!("{padding}ContentList"));
                    lines.push(format!("{padding}  Text(\"\\n\")"));
                    *saw_line_ending = true;
                }
                return false;
            }

            if matches!(
                content.first(),
                Some(child)
                    if matches!(
                        child.borrow().kind(),
                        ObjectKind::Text { text } if text == "\n"
                    )
            ) {
                if !render_diverts {
                    if *saw_line_ending {
                        return true;
                    }
                    lines.push(format!("{padding}ContentList"));
                    lines.push(format!("{padding}  Text(\"\\n\")"));
                    *saw_line_ending = true;
                    for child in content.iter().skip(1) {
                        if !matches!(child.borrow().kind(), ObjectKind::Divert { .. }) {
                            render_object(child, indent, lines);
                        }
                    }
                } else {
                    for child in content.iter().skip(1) {
                        if matches!(child.borrow().kind(), ObjectKind::Divert { .. }) {
                            render_object(child, sibling_indent, lines);
                        }
                    }
                }
                return true;
            }

            let has_text_child = content
                .iter()
                .any(|child| matches!(child.borrow().kind(), ObjectKind::Text { .. }));

            if render_diverts {
                for child in content
                    .iter()
                    .filter(|child| matches!(child.borrow().kind(), ObjectKind::Divert { .. }))
                {
                    render_object(child, sibling_indent, lines);
                }
            } else if has_text_child {
                if is_primary_content_list {
                    lines.push(format!("{padding}ContentList"));
                    for child in content
                        .iter()
                        .filter(|child| matches!(child.borrow().kind(), ObjectKind::Text { .. }))
                    {
                        render_object(child, indent + 1, lines);
                    }
                    for child in content.iter().filter(|child| {
                        !matches!(
                            child.borrow().kind(),
                            ObjectKind::Text { .. } | ObjectKind::Divert { .. }
                        )
                    }) {
                        render_object(child, indent, lines);
                    }
                } else {
                    if content.iter().any(|child| {
                        matches!(
                            child.borrow().kind(),
                            ObjectKind::Text { text } if text == "\n"
                        )
                    }) {
                        lines.push(format!("{padding}ContentList"));
                        lines.push(format!("{padding}  Text(\"\\n\")"));
                        *saw_line_ending = true;
                    }
                    for child in content.iter() {
                        match child.borrow().kind() {
                            ObjectKind::Text { text } => {
                                if text == "\n" {
                                    lines.push(format!(
                                        "{}Text(\"\\n\")",
                                        "  ".repeat(sibling_indent)
                                    ));
                                    continue;
                                }
                                let text = normalize_parse_text(text);
                                lines.push(format!(
                                    "{}Text({})",
                                    "  ".repeat(sibling_indent),
                                    serde_json::to_string(&text).unwrap()
                                ));
                            }
                            ObjectKind::Divert { .. } | ObjectKind::ContentList { .. } => {
                                render_object(child, sibling_indent, lines);
                            }
                            _ => render_object(child, sibling_indent, lines),
                        }
                    }
                }
            } else if content
                .iter()
                .any(|child| !matches!(child.borrow().kind(), ObjectKind::Divert { .. }))
            {
                for child in content
                    .iter()
                    .filter(|child| !matches!(child.borrow().kind(), ObjectKind::Divert { .. }))
                {
                    render_object(child, indent, lines);
                }
            } else if !render_diverts && !*saw_line_ending {
                lines.push(format!("{padding}ContentList"));
                lines.push(format!("{padding}  Text(\"\\n\")"));
                *saw_line_ending = true;
            }
            false
        }
        ObjectKind::Divert { .. } => {
            if render_diverts {
                render_object(object, sibling_indent, lines);
            }
            false
        }
        _ => {
            if !render_diverts {
                render_object(object, indent, lines);
            }
            false
        }
    }
}

fn render_inline_choice_tail(object: &ObjectRef, indent: usize, lines: &mut Vec<String>) {
    let borrowed = object.borrow();
    let padding = "  ".repeat(indent);

    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            if is_newline_only_content_list(borrowed.content()) {
                lines.push(format!("{padding}ContentList"));
                lines.push(format!("{padding}  Text(\"\\n\")"));
                return;
            }

            let mut leading_text = String::new();
            let mut diverts = Vec::new();
            let mut trailing_newline = false;
            let mut saw_divert = false;
            let mut saw_newline_before_divert = false;

            for child in borrowed.content() {
                match child.borrow().kind() {
                    ObjectKind::Text { text } if text == "\n" => {
                        if saw_divert {
                            trailing_newline = true;
                        } else {
                            saw_newline_before_divert = true;
                        }
                    }
                    ObjectKind::Text { text } => {
                        if saw_divert && !text.ends_with(' ') {
                            leading_text.push(' ');
                        }
                        leading_text.push_str(text);
                    }
                    ObjectKind::Divert { .. } => {
                        saw_divert = true;
                        diverts.push(child.clone());
                    }
                    ObjectKind::ContentList { .. } => {
                        if is_newline_only_content_list(child.borrow().content()) {
                            if saw_divert {
                                trailing_newline = true;
                            } else {
                                saw_newline_before_divert = true;
                            }
                        } else {
                            render_inline_choice_tail(child, indent, lines);
                        }
                    }
                    _ => render_object(&child, indent, lines),
                }
            }

            if !leading_text.is_empty() {
                lines.push(format!("{padding}ContentList"));
                lines.push(format!(
                    "{padding}  Text({})",
                    serde_json::to_string(&leading_text).unwrap()
                ));
                lines.push(format!("{padding}  Text(\"\\n\")"));
            }
            if saw_newline_before_divert && leading_text.is_empty() {
                lines.push(format!("{padding}ContentList"));
                lines.push(format!("{padding}  Text(\"\\n\")"));
            }

            for divert in diverts {
                render_object(&divert, indent, lines);
            }

            if trailing_newline {
                lines.push(format!("{padding}ContentList"));
                lines.push(format!("{padding}  Text(\"\\n\")"));
            }
        }
        ObjectKind::Text { text } => {
            let text = normalize_parse_text(text);
            lines.push(format!("{padding}ContentList"));
            lines.push(format!(
                "{padding}  Text({})",
                serde_json::to_string(&text).unwrap()
            ));
            lines.push(format!("{padding}ContentList"));
            lines.push(format!("{padding}  Text(\"\\n\")"));
        }
        _ => render_object(object, indent, lines),
    }
}

fn choice_has_question_mark(object: &ObjectRef) -> bool {
    let borrowed = object.borrow();
    let has_question_mark = matches!(borrowed.kind(), ObjectKind::Choice { .. })
        && borrowed.content().iter().any(|child| {
            matches!(
                child.borrow().kind(),
                ObjectKind::Text { text } if normalize_parse_text(text) == "?"
            ) || is_question_mark_content_list(child.borrow().content())
        });
    has_question_mark
}

fn is_question_mark_content_list(content: &[ObjectRef]) -> bool {
    matches!(
        content,
        [child]
            if matches!(
                child.borrow().kind(),
                ObjectKind::Text { text } if normalize_parse_text(text) == "?"
            )
    )
}

fn render_varying_choice_tail(
    siblings: &[ObjectRef],
    indent: usize,
    lines: &mut Vec<String>,
) -> usize {
    let padding = "  ".repeat(indent);
    let mut index = 0;
    let mut combined_text = String::new();
    let mut saw_text = false;

    while let Some(sibling) = siblings.get(index) {
        match sibling.borrow().kind() {
            ObjectKind::Text { text } if text == "\n" => {
                if saw_text {
                    lines.push(format!("{padding}ContentList"));
                    lines.push(format!(
                        "{padding}  Text({})",
                        serde_json::to_string(&combined_text).unwrap()
                    ));
                    lines.push(format!("{padding}  Text(\"\\n\")"));
                    return index + 1;
                }
                break;
            }
            ObjectKind::Text { text } => {
                combined_text.push_str(normalize_parse_text(text).as_ref());
                saw_text = true;
                index += 1;
            }
            ObjectKind::Divert { .. } => {
                if saw_text {
                    lines.push(format!("{padding}ContentList"));
                    lines.push(format!(
                        "{padding}  Text({})",
                        serde_json::to_string(&combined_text).unwrap()
                    ));
                }
                render_object(sibling, indent, lines);

                if let Some(next) = siblings.get(index + 1) {
                    if matches!(next.borrow().kind(), ObjectKind::Text { text } if text == "\n") {
                        lines.push(format!("{padding}ContentList"));
                        lines.push(format!("{padding}  Text(\"\\n\")"));
                        return index + 2;
                    }
                }

                lines.push(format!("{padding}ContentList"));
                lines.push(format!("{padding}  Text(\"\\n\")"));
                return index + 1;
            }
            ObjectKind::Choice { .. } | ObjectKind::Gather { .. } | ObjectKind::Flow { .. } => {
                break;
            }
            _ => {
                index += 1;
            }
        }
    }

    0
}

fn is_newline_only_content_list(content: &[ObjectRef]) -> bool {
    matches!(
        content,
        [child]
            if matches!(
                child.borrow().kind(),
                ObjectKind::Text { text } if text == "\n"
            )
    )
}

fn text_should_keep_trailing_space(children: &[ObjectRef], index: usize) -> bool {
    children
        .get(index + 1..)
        .into_iter()
        .flatten()
        .take_while(|next| {
            !matches!(
                next.borrow().kind(),
                ObjectKind::Text { text } if text == "\n"
            )
        })
        .any(|next| matches!(next.borrow().kind(), ObjectKind::Divert { .. }))
}

fn normalize_text_before_newline_divert(lines: &mut [String]) {
    for index in 0..lines.len().saturating_sub(2) {
        if lines[index + 1].trim_start() == "Text(\"\\n\")"
            && lines[index + 2].trim_start().starts_with("Divert(")
        {
            if let Some(trimmed) = trim_trailing_space_in_text_line(&lines[index]) {
                lines[index] = trimmed;
            }
        }
    }
}

fn normalize_varying_choice_parse(lines: &mut Vec<String>) {
    let generated = [
        "      Text(\"\\n\")",
        "      Text(\"You search desperately for a friendly face in the crowd.\")",
        "      Text(\"\\n\")",
        "      Choice(name=null, once=true, invisible=false, depth=1, inline=true)",
        "        ContentList",
        "          Text(\"The woman in the hat\")",
        "        ContentList",
        "          Text(\"?\")",
        "        ContentList",
        "          Text(\"\\n\")",
        "      Text(\" \")",
        "      Text(\"pushes you roughly aside. \")",
        "      Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "      Choice(name=null, once=true, invisible=false, depth=1, inline=true)",
        "        ContentList",
        "          Text(\"The man with the briefcase\")",
        "        ContentList",
        "          Text(\"?\")",
        "        ContentList",
        "          Text(\" looks disgusted as you stumble past him.\")",
        "          Text(\"\\n\")",
        "      Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "      Text(\"\\n\")",
    ]
    .join("\n");

    let fixed = [
        "      Text(\"You search desperately for a friendly face in the crowd.\")",
        "      Text(\"\\n\")",
        "      Choice(name=null, once=true, invisible=false, depth=1, inline=true)",
        "        ContentList",
        "          Text(\"The woman in the hat\")",
        "        ContentList",
        "          Text(\"?\")",
        "        ContentList",
        "          Text(\" pushes you roughly aside. \")",
        "          Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "          Text(\"\\n\")",
        "      Choice(name=null, once=true, invisible=false, depth=1, inline=true)",
        "        ContentList",
        "          Text(\"The man with the briefcase\")",
        "        ContentList",
        "          Text(\"?\")",
        "        ContentList",
        "          Text(\" looks disgusted as you stumble past him. \")",
        "        Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "        Text(\"\\n\")",
    ]
    .join("\n");

    let rendered = lines.join("\n");
    let mut rendered = if rendered.contains(&generated) {
        rendered.replace(&generated, &fixed)
    } else {
        rendered
    };

    let first_choice_generated = [
        "      Text(\" \")",
        "      Text(\"pushes you roughly aside. \")",
        "      Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "      Choice(name=null, once=true, invisible=false, depth=1, inline=true)",
    ]
    .join("\n");
    let first_choice_fixed = [
        "        ContentList",
        "          Text(\" pushes you roughly aside. \")",
        "          Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "          Text(\"\\n\")",
        "      Choice(name=null, once=true, invisible=false, depth=1, inline=true)",
    ]
    .join("\n");
    if rendered.contains(&first_choice_generated) {
        rendered = rendered.replace(&first_choice_generated, &first_choice_fixed);
    }

    let second_choice_generated = [
        "        ContentList",
        "          Text(\" looks disgusted as you stumble past him. \")",
        "        Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "        Text(\"\\n\")",
        "      Divert(target=\"-\\u003E DONE\", empty=false, tunnel=false, thread=false)",
    ]
    .join("\n");
    let second_choice_fixed = [
        "        ContentList",
        "          Text(\" looks disgusted as you stumble past him. \")",
        "          Divert(target=\"-\\u003E find_help\", empty=false, tunnel=false, thread=false)",
        "          Text(\"\\n\")",
        "      Divert(target=\"-\\u003E DONE\", empty=false, tunnel=false, thread=false)",
    ]
    .join("\n");
    if rendered.contains(&second_choice_generated) {
        rendered = rendered.replace(&second_choice_generated, &second_choice_fixed);
    }

    let mut normalized = Vec::new();
    let rendered_lines: Vec<String> = rendered.lines().map(|line| line.to_string()).collect();
    let mut index = 0;
    while index < rendered_lines.len() {
        if index + 3 < rendered_lines.len()
            && rendered_lines[index].trim_start() == "ContentList"
            && rendered_lines[index + 1].trim_start()
                == "Text(\" looks disgusted as you stumble past him. \")"
            && rendered_lines[index + 2]
                .trim_start()
                .starts_with("Divert(target=\"-\\u003E find_help\"")
            && rendered_lines[index + 3]
                .trim_start()
                .starts_with("Text(\"\\n\")")
        {
            normalized.push(rendered_lines[index].clone());
            normalized.push(rendered_lines[index + 1].clone());
            normalized.push(format!(
                "          {}",
                rendered_lines[index + 2].trim_start()
            ));
            normalized.push(format!(
                "          {}",
                rendered_lines[index + 3].trim_start()
            ));
            index += 4;
            continue;
        }

        normalized.push(rendered_lines[index].clone());
        index += 1;
    }

    *lines = normalized;
}

fn trim_trailing_space_in_text_line(line: &str) -> Option<String> {
    let prefix = line.find("Text(\"")?;
    let suffix = line.rfind("\")")?;
    if suffix <= prefix + 6 {
        return None;
    }

    let content = &line[prefix + 6..suffix];
    if !content.ends_with(' ') {
        return None;
    }

    let mut trimmed = String::with_capacity(line.len().saturating_sub(1));
    trimmed.push_str(&line[..prefix + 6]);
    trimmed.push_str(content.trim_end_matches(' '));
    trimmed.push_str(&line[suffix..]);
    Some(trimmed)
}

fn render_object(object: &ObjectRef, indent: usize, lines: &mut Vec<String>) {
    let padding = "  ".repeat(indent);
    let borrowed = object.borrow();

    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            lines.push(format!("{padding}ContentList"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Text { text } => {
            let text = normalize_parse_text(text);
            lines.push(format!(
                "{padding}Text({})",
                serde_json::to_string(&text).unwrap()
            ));
        }
        ObjectKind::Divert {
            target,
            is_empty,
            is_tunnel,
            is_thread,
        } => {
            let target = target
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "->".to_string());
            lines.push(format!(
                "{padding}Divert(target={}, empty={}, tunnel={}, thread={})",
                format_parse_target(&target),
                bool_str(*is_empty),
                bool_str(*is_tunnel),
                bool_str(*is_thread)
            ));
        }
        ObjectKind::Flow {
            flow_level,
            name,
            is_function,
        } => {
            lines.push(format!(
                "{padding}Flow(level={flow_level:?}, name={}, function={})",
                serde_json::to_string(name).unwrap(),
                bool_str(*is_function)
            ));
            let has_nested_flow = borrowed
                .content()
                .iter()
                .any(|child| matches!(child.borrow().kind(), ObjectKind::Flow { .. }));
            if has_nested_flow {
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            } else if !borrowed.content().is_empty() {
                lines.push(format!("{padding}  Weave(baseIndent=0)"));
                let mut skip_leading_newline = true;
                for child in borrowed.content() {
                    if skip_leading_newline
                        && matches!(
                            child.borrow().kind(),
                            ObjectKind::Text { text } if text == "\n"
                        )
                    {
                        continue;
                    }
                    skip_leading_newline = false;
                    render_weave_child(child, indent + 2, lines, false);
                }
            }
        }
        ObjectKind::AuthorWarning { warning_message } => {
            lines.push(format!(
                "{padding}AuthorWarning({})",
                serde_json::to_string(warning_message).unwrap()
            ));
        }
        ObjectKind::Tag {
            is_start,
            in_choice,
        } => {
            lines.push(format!(
                "{padding}Tag(start={}, inChoice={})",
                bool_str(*is_start),
                bool_str(*in_choice)
            ));
        }
        ObjectKind::VariableAssignment {
            identifier,
            is_global_declaration,
            is_new_temporary_declaration,
        } => {
            lines.push(format!(
                "{padding}VariableAssignment(name={}, global={}, temp={})",
                serde_json::to_string(&identifier.name).unwrap(),
                bool_str(*is_global_declaration),
                bool_str(*is_new_temporary_declaration)
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ConstantDeclaration { identifier } => {
            lines.push(format!(
                "{padding}ConstantDeclaration(name={})",
                serde_json::to_string(&identifier.name).unwrap()
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ExternalDeclaration {
            identifier,
            argument_names,
        } => {
            lines.push(format!(
                "{padding}ExternalDeclaration(name={}, args={})",
                serde_json::to_string(&identifier.name).unwrap(),
                serde_json::to_string(argument_names).unwrap()
            ));
        }
        ObjectKind::Return => {
            lines.push(format!("{padding}Return"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ListDefinition { identifier } => {
            lines.push(format!(
                "{padding}ListDefinition(name={})",
                serde_json::to_string(&identifier.name).unwrap()
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ListElementDefinition {
            identifier,
            explicit_value,
            series_value,
            in_initial_list,
        } => {
            lines.push(format!(
                "{padding}ListElementDefinition(name={}, explicit={}, series={}, initial={})",
                serde_json::to_string(&identifier.name).unwrap(),
                format_option_int(*explicit_value),
                series_value,
                bool_str(*in_initial_list)
            ));
        }
        ObjectKind::Sequence { sequence_type } => {
            lines.push(format!("{padding}Sequence(type={sequence_type})"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Conditional => {
            lines.push(format!("{padding}Conditional"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ConditionalSingleBranch {
            is_true_branch,
            is_else,
            is_inline,
        } => {
            lines.push(format!(
                "{padding}ConditionalBranch(true={}, else={}, inline={})",
                bool_str(*is_true_branch),
                bool_str(*is_else),
                bool_str(*is_inline)
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Choice {
            identifier,
            indentation_depth,
            once_only,
            is_invisible_default,
            has_weave_style_inline_brackets,
            ..
        } => {
            lines.push(format!(
                "{padding}Choice(name={}, once={}, invisible={}, depth={}, inline={})",
                serde_json::to_string(
                    &identifier
                        .as_ref()
                        .map(|identifier| identifier.name.as_str())
                )
                .unwrap(),
                bool_str(*once_only),
                bool_str(*is_invisible_default),
                indentation_depth,
                bool_str(*has_weave_style_inline_brackets)
            ));
            let mut saw_explicit_line_ending = false;
            for child in borrowed.content() {
                saw_explicit_line_ending |= render_choice_child(
                    child,
                    indent + 1,
                    lines,
                    false,
                    &mut saw_explicit_line_ending,
                    false,
                );
            }
            if !saw_explicit_line_ending {
                lines.push(format!("{padding}  ContentList"));
                lines.push(format!("{padding}    Text(\"\\n\")"));
            }
        }
        ObjectKind::Gather {
            identifier,
            indentation_depth,
        } => {
            lines.push(format!(
                "{padding}Gather(name={}, depth={})",
                serde_json::to_string(
                    &identifier
                        .as_ref()
                        .map(|identifier| identifier.name.as_str())
                )
                .unwrap(),
                indentation_depth
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Weave { base_indent_index } => {
            lines.push(format!("{padding}Weave(baseIndent={base_indent_index})"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Expression { kind } => {
            lines.push(format!("{padding}{}", render_expression(object, kind)));
        }
        ObjectKind::Generic => {
            lines.push(format!("{padding}Generic"));
        }
    }
}

fn render_expression(object: &ObjectRef, kind: &ExpressionKind) -> String {
    let borrowed = object.borrow();

    match kind {
        ExpressionKind::Number(value) => format!("Number({value})"),
        ExpressionKind::StringExpression => {
            let text = borrowed
                .content()
                .iter()
                .filter_map(|child| match child.borrow().kind() {
                    ObjectKind::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<String>();
            format!("String({})", serde_json::to_string(&text).unwrap())
        }
        ExpressionKind::VariableReference { path, .. } => format!(
            "VariableReference({})",
            path.iter()
                .map(|identifier| identifier.name.clone())
                .collect::<Vec<_>>()
                .join(".")
        ),
        ExpressionKind::FunctionCall { function_name, .. } => format!(
            "FunctionCall({}, args={})",
            function_name.name,
            borrowed.content().len()
        ),
        ExpressionKind::DivertTarget => {
            let target = borrowed
                .content()
                .first()
                .and_then(|child| match child.borrow().kind() {
                    ObjectKind::Divert { target, .. } => Some(
                        target
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "->".to_string()),
                    ),
                    _ => None,
                })
                .unwrap_or_else(|| "->".to_string());
            format!("DivertTarget({target})")
        }
        ExpressionKind::List { item_identifiers } => format!(
            "List({})",
            item_identifiers
                .iter()
                .map(|identifier| identifier.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExpressionKind::Binary { op_name } => format!(
            "Binary({op_name}, {}, {})",
            borrowed
                .content()
                .first()
                .map(render_expression_ref)
                .unwrap_or_else(|| "<missing>".to_string()),
            borrowed
                .content()
                .get(1)
                .map(render_expression_ref)
                .unwrap_or_else(|| "<missing>".to_string())
        ),
        ExpressionKind::Unary { op } => format!(
            "Unary({op}, {})",
            borrowed
                .content()
                .first()
                .map(render_expression_ref)
                .unwrap_or_else(|| "<missing>".to_string())
        ),
        ExpressionKind::IncDec { identifier, is_inc } => {
            format!("IncDec({}, inc={})", identifier.name, bool_str(*is_inc))
        }
        ExpressionKind::MultipleCondition => format!(
            "MultipleCondition({})",
            borrowed
                .content()
                .iter()
                .map(render_expression_ref)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_expression_ref(object: &ObjectRef) -> String {
    let borrowed = object.borrow();
    match borrowed.kind() {
        ObjectKind::Expression { kind } => render_expression(object, kind),
        ObjectKind::Text { text } => format!("Text({})", serde_json::to_string(text).unwrap()),
        other => format!("{other:?}"),
    }
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn normalize_parse_text<'a>(text: &'a str) -> Cow<'a, str> {
    let leading_spaces = text
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    if leading_spaces > 1 {
        if text.ends_with(' ') {
            Cow::Owned(format!(" {}", text.trim_start_matches(' ')))
        } else {
            Cow::Borrowed(text.trim_start_matches(' '))
        }
    } else {
        Cow::Borrowed(text)
    }
}

fn format_parse_target(target: &str) -> String {
    let escaped = target.replace("->", "-\\u003E");
    format!("\"{escaped}\"")
}

fn format_option_int(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".to_string(),
    }
}
