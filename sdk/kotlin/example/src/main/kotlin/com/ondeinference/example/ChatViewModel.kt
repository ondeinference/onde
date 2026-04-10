package com.ondeinference.example

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.ondeinference.onde.OndeInference
import com.ondeinference.onde.OndeSampling
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

// One message in the chat list.
data class UiMessage(
    val id: Long = System.nanoTime(), // nanoTime as a stable list key — collisions are basically impossible
    val content: String,
    val isUser: Boolean,
    val isStreaming: Boolean = false, // true while the model is still typing
)

// Where the model is in its lifecycle.
sealed class ModelState {
    object Unloaded : ModelState()
    object Loading : ModelState()
    data class Ready(val name: String, val loadTimeSeconds: Double) : ModelState()
    data class Error(val cause: String) : ModelState()
}

// Everything the UI needs, in one place. Compose observes this and re-renders when it changes.
data class ChatUiState(
    val modelState: ModelState = ModelState.Unloaded,
    val messages: List<UiMessage> = emptyList(),
    val isGenerating: Boolean = false, // blocks the send button while a reply is in flight
    val input: String = "",
)

class ChatViewModel(app: Application) : AndroidViewModel(app) {

    // The Rust engine, wrapped by the Onde SDK. We pass applicationContext to avoid leaking the Activity.
    private val onde = OndeInference(app.applicationContext)

    private val _uiState = MutableStateFlow(ChatUiState())
    val uiState: StateFlow<ChatUiState> = _uiState.asStateFlow()

    // Pull down the model and get it ready to chat.
    // On first launch this downloads ~941 MB from HuggingFace; after that it loads straight from cache.
    fun loadModel() {
        viewModelScope.launch {
            _uiState.update { it.copy(modelState = ModelState.Loading) }

            runCatching {
                onde.loadDefaultModel(
                    systemPrompt = "You are a helpful, concise assistant " +
                        "running entirely on-device. No data leaves this phone.",
                    sampling = OndeSampling.mobile(), // shorter replies feel snappier on mobile
                )
            }.onSuccess { elapsedSeconds ->
                val info = onde.info()
                _uiState.update {
                    it.copy(
                        modelState = ModelState.Ready(
                            name = info.modelName ?: "Qwen 2.5 1.5B",
                            loadTimeSeconds = elapsedSeconds,
                        ),
                    )
                }
            }.onFailure { error ->
                _uiState.update {
                    it.copy(
                        modelState = ModelState.Error(
                            error.message ?: "Something went wrong — check logcat for details",
                        ),
                    )
                }
            }
        }
    }

    // Send the current input and stream the reply back token by token.
    fun sendMessage() {
        val text = _uiState.value.input.trim()
        if (text.isBlank() || _uiState.value.isGenerating) return

        // Show the user's message right away so the UI feels snappy.
        _uiState.update {
            it.copy(
                messages = it.messages + UiMessage(content = text, isUser = true),
                input = "",
                isGenerating = true,
            )
        }

        viewModelScope.launch {
            // Drop in an empty assistant bubble and fill it in as tokens arrive,
            // rather than appending a new item on every chunk.
            val placeholderId = System.nanoTime()
            _uiState.update {
                it.copy(
                    messages = it.messages + UiMessage(
                        id = placeholderId,
                        content = "",
                        isUser = false,
                        isStreaming = true,
                    ),
                )
            }

            val buffer = StringBuilder()

            onde.stream(text)
                .catch { error ->
                    // Something broke mid-stream — show the error inside the bubble instead of crashing.
                    _uiState.update { state ->
                        state.copy(
                            messages = state.messages.replaceLast(
                                UiMessage(
                                    id = placeholderId,
                                    content = "⚠ ${error.message ?: "Inference failed"}",
                                    isUser = false,
                                    isStreaming = false,
                                ),
                            ),
                            isGenerating = false,
                        )
                    }
                }
                .collect { chunk ->
                    buffer.append(chunk.delta)
                    _uiState.update { state ->
                        state.copy(
                            messages = state.messages.replaceLast(
                                UiMessage(
                                    id = placeholderId,
                                    content = buffer.toString(),
                                    isUser = false,
                                    isStreaming = !chunk.done,
                                ),
                            ),
                            isGenerating = !chunk.done,
                        )
                    }
                }
        }
    }

    fun updateInput(text: String) {
        _uiState.update { it.copy(input = text) }
    }

    // Wipe the screen and reset the model's memory of the conversation.
    // The model stays loaded — no re-download needed.
    fun clearChat() {
        viewModelScope.launch {
            onde.clearHistory()
            _uiState.update { it.copy(messages = emptyList()) }
        }
    }

    // Clean up when the screen is gone for good.
    override fun onCleared() {
        super.onCleared()
        viewModelScope.launch {
            onde.unload()
            onde.close()
        }
    }

    // Swap the last item in the list in-place — keeps the streaming bubble updating smoothly.
    private fun List<UiMessage>.replaceLast(replacement: UiMessage): List<UiMessage> =
        if (isEmpty()) listOf(replacement) else dropLast(1) + replacement
}
