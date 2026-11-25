// Copyright 2026 Seto Elkahfi (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pure text helpers for the browser engine: ChatML prompt building. Nothing
//! here touches vectoron or JS, so the unit tests run on the host.

use super::types::{ChatMessage, ChatRole};

/// Build a ChatML prompt (Qwen's template) ending with an open assistant turn.
///
/// `system` goes first when present. `messages` may itself start with a system
/// message; that one wins over `system`, so `generate` callers keep full
/// control of the prompt.
pub(crate) fn chatml_prompt(system: Option<&str>, messages: &[ChatMessage]) -> String {
    let mut out = String::new();
    let leading_system = matches!(messages.first(), Some(m) if m.role == ChatRole::System);
    if let Some(system) = system.filter(|_| !leading_system) {
        push_turn(&mut out, "system", system);
    }
    for message in messages {
        push_turn(&mut out, &message.role.to_string(), &message.content);
    }
    out.push_str("<|im_start|>assistant\n");
    out
}

fn push_turn(out: &mut String, role: &str, content: &str) {
    out.push_str("<|im_start|>");
    out.push_str(role);
    out.push('\n');
    out.push_str(content);
    out.push_str("<|im_end|>\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_with_system_history_and_user() {
        let history = [
            ChatMessage::user("Hi"),
            ChatMessage::assistant("Hello!"),
            ChatMessage::user("Name a colour."),
        ];
        assert_eq!(
            chatml_prompt(Some("Be brief."), &history),
            "<|im_start|>system\nBe brief.<|im_end|>\n\
             <|im_start|>user\nHi<|im_end|>\n\
             <|im_start|>assistant\nHello!<|im_end|>\n\
             <|im_start|>user\nName a colour.<|im_end|>\n\
             <|im_start|>assistant\n"
        );
    }

    #[test]
    fn prompt_without_system() {
        assert_eq!(
            chatml_prompt(None, &[ChatMessage::user("Hi")]),
            "<|im_start|>user\nHi<|im_end|>\n<|im_start|>assistant\n"
        );
    }

    #[test]
    fn leading_system_message_wins() {
        let messages = [ChatMessage::system("Mine."), ChatMessage::user("Hi")];
        let prompt = chatml_prompt(Some("Default."), &messages);
        assert!(prompt.contains("Mine."));
        assert!(!prompt.contains("Default."));
    }
}
