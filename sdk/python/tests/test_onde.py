"""Tests for onde-inference Python bindings.

These tests verify the UniFFI-generated Python bindings work correctly.
They test the public API surface without requiring model downloads or
GPU hardware — focusing on construction, configuration, and error handling.
"""
import pytest

# All imports should come from the public package
from onde_inference import (
    OndeChatEngine,
    ChatMessage,
    ChatRole,
    SamplingConfig,
    GgufModelConfig,
    EngineInfo,
    EngineStatus,
    InferenceError,
    StreamChunk,
    # Free functions
    default_model_config,
    qwen25_1_5b_config,
    qwen25_3b_config,
    default_sampling_config,
    deterministic_sampling_config,
    mobile_sampling_config,
    system_message,
    user_message,
    assistant_message,
    # Version
    __version__,
)


class TestVersion:
    def test_version_is_string(self):
        assert isinstance(__version__, str)

    def test_version_is_semver(self):
        parts = __version__.split(".")
        assert len(parts) == 3
        assert all(p.isdigit() for p in parts)


class TestModelConfigs:
    def test_default_model_config_not_empty(self):
        cfg = default_model_config()
        assert isinstance(cfg, GgufModelConfig)
        assert cfg.model_id != ""
        assert len(cfg.files) > 0
        assert cfg.display_name != ""

    def test_qwen25_1_5b_config(self):
        cfg = qwen25_1_5b_config()
        assert "1.5B" in cfg.model_id
        assert len(cfg.files) == 1
        assert cfg.files[0].endswith(".gguf")

    def test_qwen25_3b_config(self):
        cfg = qwen25_3b_config()
        assert "3B" in cfg.model_id
        assert len(cfg.files) == 1
        assert cfg.files[0].endswith(".gguf")


class TestSamplingConfigs:
    def test_default_sampling(self):
        s = default_sampling_config()
        assert isinstance(s, SamplingConfig)
        assert s.temperature == 0.7
        assert s.top_p == 0.95
        assert s.max_tokens == 512

    def test_deterministic_sampling(self):
        s = deterministic_sampling_config()
        assert s.temperature == 0.0
        assert s.top_p is None

    def test_mobile_sampling(self):
        s = mobile_sampling_config()
        assert s.max_tokens == 128


class TestMessageHelpers:
    def test_system_message(self):
        m = system_message("You are helpful.")
        assert isinstance(m, ChatMessage)
        assert m.role == ChatRole.SYSTEM
        assert m.content == "You are helpful."

    def test_user_message(self):
        m = user_message("Hello")
        assert m.role == ChatRole.USER
        assert m.content == "Hello"

    def test_assistant_message(self):
        m = assistant_message("Hi!")
        assert m.role == ChatRole.ASSISTANT
        assert m.content == "Hi!"


class TestChatRole:
    def test_enum_variants_exist(self):
        assert ChatRole.SYSTEM is not None
        assert ChatRole.USER is not None
        assert ChatRole.ASSISTANT is not None


class TestEngineStatus:
    def test_enum_variants_exist(self):
        assert EngineStatus.UNLOADED is not None
        assert EngineStatus.LOADING is not None
        assert EngineStatus.READY is not None
        assert EngineStatus.GENERATING is not None
        assert EngineStatus.ERROR is not None


@pytest.mark.asyncio
class TestOndeChatEngine:
    async def test_new_engine(self):
        engine = OndeChatEngine()
        assert engine is not None

    async def test_engine_not_loaded_initially(self):
        engine = OndeChatEngine()
        assert await engine.is_loaded() is False

    async def test_engine_info_initial_state(self):
        engine = OndeChatEngine()
        info = await engine.info()
        assert isinstance(info, EngineInfo)
        assert info.status == EngineStatus.UNLOADED
        assert info.history_length == 0
        assert info.model_name is None

    async def test_send_message_without_model_raises(self):
        engine = OndeChatEngine()
        with pytest.raises(InferenceError):
            await engine.send_message("hello")

    async def test_history_empty_initially(self):
        engine = OndeChatEngine()
        history = await engine.history()
        assert history == []

    async def test_clear_history_returns_zero(self):
        engine = OndeChatEngine()
        removed = await engine.clear_history()
        assert removed == 0

    async def test_unload_when_none_returns_none(self):
        engine = OndeChatEngine()
        result = await engine.unload_model()
        assert result is None