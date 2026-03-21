"""onde-inference — lightweight LLM inference powered by Rust.

This package wraps the native Rust ``onde`` crate via UniFFI bindings that are
generated at build time by *maturin*.  The native extension module lives at
``onde_inference._native``; everything useful is re-exported here so you can
simply write::

    from onde_inference import OndeChatEngine, user_message, ChatRole

Public API
----------
Classes
    OndeChatEngine
Enums
    ChatRole, EngineStatus
Records (dataclasses)
    ChatMessage, SamplingConfig, GgufModelConfig, InferenceResult,
    StreamChunk, EngineInfo
Errors
    InferenceError
Callback interfaces (protocols)
    StreamChunkListener
Free functions
    stream_chat_message, default_model_config, qwen25_1_5b_config,
    qwen25_3b_config, default_sampling_config,
    deterministic_sampling_config, mobile_sampling_config,
    system_message, user_message, assistant_message
"""

from onde_inference._native import *  # noqa: F401,F403
from onde_inference._native import (
    # Classes
    OndeChatEngine,
    # Enums
    ChatRole,
    EngineStatus,
    # Records (dataclasses)
    ChatMessage,
    SamplingConfig,
    GgufModelConfig,
    InferenceResult,
    StreamChunk,
    EngineInfo,
    # Errors
    InferenceError,
    # Callback interfaces (protocols)
    StreamChunkListener,
    # Free functions
    stream_chat_message,
    default_model_config,
    qwen25_1_5b_config,
    qwen25_3b_config,
    default_sampling_config,
    deterministic_sampling_config,
    mobile_sampling_config,
    system_message,
    user_message,
    assistant_message,
)

__version__ = "0.1.0"

__all__ = [
    # Metadata
    "__version__",
    # Classes
    "OndeChatEngine",
    # Enums
    "ChatRole",
    "EngineStatus",
    # Records (dataclasses)
    "ChatMessage",
    "SamplingConfig",
    "GgufModelConfig",
    "InferenceResult",
    "StreamChunk",
    "EngineInfo",
    # Errors
    "InferenceError",
    # Callback interfaces (protocols)
    "StreamChunkListener",
    # Free functions
    "stream_chat_message",
    "default_model_config",
    "qwen25_1_5b_config",
    "qwen25_3b_config",
    "default_sampling_config",
    "deterministic_sampling_config",
    "mobile_sampling_config",
    "system_message",
    "user_message",
    "assistant_message",
]