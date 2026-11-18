// Copyright 2026 Seto Elkahfi (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pure text helpers for the browser engine: ChatML prompt building and
//! logit penalties. Nothing here touches candle or JS, so the unit tests run
//! on the host.

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

/// Apply OpenAI-style frequency and presence penalties in place.
///
/// Each token already generated lowers its own logit by
/// `frequency * count + presence`.
pub(crate) fn apply_penalties(
    logits: &mut [f32],
    generated: &[u32],
    frequency: f32,
    presence: f32,
) {
    if frequency == 0.0 && presence == 0.0 {
        return;
    }
    let mut counts = std::collections::HashMap::<u32, u32>::new();
    for &token in generated {
        *counts.entry(token).or_default() += 1;
    }
    for (token, count) in counts {
        if let Some(logit) = logits.get_mut(token as usize) {
            *logit -= frequency * count as f32 + presence;
        }
    }
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

    #[test]
    fn penalties_lower_seen_tokens_only() {
        let mut logits = [1.0_f32; 4];
        apply_penalties(&mut logits, &[1, 1, 3], 0.5, 0.25);
        assert_eq!(logits, [1.0, 1.0 - (1.0 + 0.25), 1.0, 1.0 - (0.5 + 0.25)]);
    }

    #[test]
    fn zero_penalties_are_a_no_op() {
        let mut logits = [2.0_f32, 3.0];
        apply_penalties(&mut logits, &[0, 1], 0.0, 0.0);
        assert_eq!(logits, [2.0, 3.0]);
    }

    #[test]
    fn out_of_range_tokens_are_ignored() {
        let mut logits = [1.0_f32];
        apply_penalties(&mut logits, &[99], 1.0, 1.0);
        assert_eq!(logits, [1.0]);
    }
}
