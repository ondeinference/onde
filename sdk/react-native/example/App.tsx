import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  FlatList,
  ActivityIndicator,
  SafeAreaView,
  KeyboardAvoidingView,
  Platform,
  Alert,
  StyleSheet,
} from "react-native";
import {
  OndeChatEngine,
  userMessage,
  type ChatMessage,
  type EngineStatus,
} from "@ondeinference/react-native";

// ── Onde app credentials ─────────────────────────────────────────────────
// Register your app at https://ondeinference.com to get these.
// The dashboard lets you assign a model to your app — the SDK fetches
// it automatically. If no model is assigned, the platform default is used.
const ONDE_APP_ID = "";
const ONDE_APP_SECRET = "";

interface DisplayMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  duration?: string;
}

const STATUS_COLORS: Record<EngineStatus, string> = {
  Unloaded: "#999",
  Loading: "#FF9500",
  Ready: "#34C759",
  Generating: "#007AFF",
  Error: "#FF3B30",
};

const App: React.FC = () => {
  const [messages, setMessages] = useState<DisplayMessage[]>([]);
  const [inputText, setInputText] = useState("");
  const [engineStatus, setEngineStatus] = useState<EngineStatus>("Unloaded");
  const [modelName, setModelName] = useState<string | null>(null);
  const [approxMemory, setApproxMemory] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);

  const normalizeEngineStatus = useCallback((status: string): EngineStatus => {
    switch (status.toLowerCase()) {
      case "loading":
        return "Loading";
      case "ready":
        return "Ready";
      case "generating":
        return "Generating";
      case "error":
        return "Error";
      case "unloaded":
      default:
        return "Unloaded";
    }
  }, []);

  const refreshStatus = useCallback(async () => {
    try {
      const info = await OndeChatEngine.info();
      setEngineStatus(normalizeEngineStatus(info.status));
      setModelName(info.modelName ?? null);
      setApproxMemory(info.approxMemory ?? null);
    } catch {
      /* status refresh is best-effort */
    }
  }, [normalizeEngineStatus]);

  const handleLoadModel = useCallback(async () => {
    setIsLoading(true);
    setEngineStatus("Loading");
    try {
      let seconds: number;
      if (ONDE_APP_ID && ONDE_APP_SECRET) {
        seconds = await OndeChatEngine.loadAssignedModel(
          ONDE_APP_ID,
          ONDE_APP_SECRET,
          "You are a helpful, concise assistant.",
        );
      } else {
        seconds = await OndeChatEngine.loadDefaultModel(
          "You are a helpful, concise assistant.",
        );
      }
      await refreshStatus();
      Alert.alert("Model Loaded", `Ready in ${seconds.toFixed(1)}s`);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : "Unknown error";
      Alert.alert("Load Failed", message);
      setEngineStatus("Unloaded");
    } finally {
      setIsLoading(false);
    }
  }, [refreshStatus]);

  const handleUnloadModel = useCallback(async () => {
    try {
      const name = await OndeChatEngine.unloadModel();
      setMessages([]);
      setEngineStatus("Unloaded");
      setModelName(null);
      setApproxMemory(null);
      if (name) {
        Alert.alert("Unloaded", `${name} has been unloaded.`);
      }
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : "Unknown error";
      Alert.alert("Unload Failed", message);
    }
  }, []);

  const handleSendMessage = useCallback(async () => {
    const text = inputText.trim();
    if (!text || !OndeChatEngine.isLoaded()) return;

    const userMsg: DisplayMessage = {
      id: Date.now().toString(),
      role: "user",
      content: text,
    };
    setMessages((previous) => [...previous, userMsg]);
    setInputText("");
    setIsGenerating(true);
    setEngineStatus("Generating");

    try {
      const result = await OndeChatEngine.sendMessage(text);
      const assistantMsg: DisplayMessage = {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: result.text,
        duration: result.durationDisplay,
      };
      setMessages((previous) => [...previous, assistantMsg]);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : "Unknown error";
      Alert.alert("Inference Error", message);
    } finally {
      setIsGenerating(false);
      await refreshStatus();
    }
  }, [inputText, refreshStatus]);

  const handleClearHistory = useCallback(() => {
    const removed = OndeChatEngine.clearHistory();
    setMessages([]);
    Alert.alert("History Cleared", `Removed ${removed} message(s).`);
  }, []);

  const renderMessage = useCallback(({ item }: { item: DisplayMessage }) => {
    const isUser = item.role === "user";
    return (
      <View
        style={[
          styles.messageBubble,
          isUser ? styles.userBubble : styles.assistantBubble,
        ]}
      >
        <Text style={isUser ? styles.userText : styles.assistantText}>
          {item.content}
        </Text>
        {item.duration && (
          <Text style={styles.durationText}>{item.duration}</Text>
        )}
      </View>
    );
  }, []);

  const isLoaded = engineStatus === "Ready" || engineStatus === "Generating";
  const canSend = isLoaded && inputText.trim().length > 0 && !isGenerating;

  return (
    <SafeAreaView style={styles.container}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        {/* Header */}
        <View style={styles.header}>
          <View style={styles.headerRow}>
            <Text style={styles.title}>Onde</Text>
            <View style={styles.statusBadge}>
              <View
                style={[
                  styles.statusDot,
                  { backgroundColor: STATUS_COLORS[engineStatus] },
                ]}
              />
              <Text style={styles.statusText}>{engineStatus}</Text>
            </View>
          </View>
          {modelName && (
            <Text style={styles.modelInfo}>
              {modelName}
              {approxMemory ? ` · ${approxMemory}` : ""}
            </Text>
          )}
        </View>

        {/* Message List */}
        <FlatList
          data={messages}
          renderItem={renderMessage}
          keyExtractor={(item) => item.id}
          style={styles.messageList}
          contentContainerStyle={styles.messageListContent}
          keyboardShouldPersistTaps="handled"
        />

        {/* Generating Indicator */}
        {isGenerating && (
          <View style={styles.generatingRow}>
            <ActivityIndicator size="small" color="#007AFF" />
            <Text style={styles.generatingText}>Generating…</Text>
          </View>
        )}

        {/* Bottom Controls */}
        <View style={styles.bottomArea}>
          <TouchableOpacity
            style={[styles.loadButton, isLoaded && styles.unloadButton]}
            onPress={isLoaded ? handleUnloadModel : handleLoadModel}
            disabled={isLoading}
          >
            {isLoading ? (
              <ActivityIndicator size="small" color="#fff" />
            ) : (
              <Text style={styles.loadButtonText}>
                {isLoaded ? "Unload" : "Load Model"}
              </Text>
            )}
          </TouchableOpacity>

          <View style={styles.inputRow}>
            <TextInput
              style={styles.textInput}
              value={inputText}
              onChangeText={setInputText}
              placeholder={isLoaded ? "Type a message…" : "Load a model first"}
              placeholderTextColor="#999"
              editable={isLoaded}
              returnKeyType="send"
              onSubmitEditing={canSend ? handleSendMessage : undefined}
            />
            <TouchableOpacity
              style={[styles.sendButton, !canSend && styles.sendButtonDisabled]}
              onPress={handleSendMessage}
              disabled={!canSend}
            >
              <Text style={styles.sendButtonText}>Send</Text>
            </TouchableOpacity>
          </View>

          {messages.length > 0 && (
            <TouchableOpacity
              style={styles.clearButton}
              onPress={handleClearHistory}
            >
              <Text style={styles.clearButtonText}>Clear History</Text>
            </TouchableOpacity>
          )}
        </View>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#fff" },
  flex: { flex: 1 },

  header: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: "#ddd",
  },
  headerRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  title: { fontSize: 20, fontWeight: "700", color: "#000" },
  statusBadge: { flexDirection: "row", alignItems: "center", gap: 6 },
  statusDot: { width: 8, height: 8, borderRadius: 4 },
  statusText: { fontSize: 13, color: "#666" },
  modelInfo: { fontSize: 12, color: "#999", marginTop: 4 },

  messageList: { flex: 1 },
  messageListContent: { padding: 16, gap: 10 },
  messageBubble: {
    maxWidth: "80%",
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 16,
  },
  userBubble: {
    alignSelf: "flex-end",
    backgroundColor: "#007AFF",
    borderBottomRightRadius: 4,
  },
  assistantBubble: {
    alignSelf: "flex-start",
    backgroundColor: "#F0F0F0",
    borderBottomLeftRadius: 4,
  },
  userText: { color: "#fff", fontSize: 15 },
  assistantText: { color: "#000", fontSize: 15 },
  durationText: { fontSize: 11, color: "#888", marginTop: 4 },

  generatingRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    paddingHorizontal: 16,
    paddingVertical: 6,
  },
  generatingText: { fontSize: 13, color: "#007AFF" },

  bottomArea: {
    padding: 12,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: "#ddd",
    gap: 10,
  },
  loadButton: {
    backgroundColor: "#007AFF",
    paddingVertical: 12,
    borderRadius: 10,
    alignItems: "center",
  },
  unloadButton: { backgroundColor: "#FF3B30" },
  loadButtonText: { color: "#fff", fontSize: 16, fontWeight: "600" },

  inputRow: { flexDirection: "row", gap: 8 },
  textInput: {
    flex: 1,
    borderWidth: StyleSheet.hairlineWidth,
    borderColor: "#ccc",
    borderRadius: 10,
    paddingHorizontal: 14,
    paddingVertical: 10,
    fontSize: 15,
    color: "#000",
    backgroundColor: "#FAFAFA",
  },
  sendButton: {
    backgroundColor: "#007AFF",
    paddingHorizontal: 18,
    borderRadius: 10,
    justifyContent: "center",
  },
  sendButtonDisabled: { backgroundColor: "#B0D4FF" },
  sendButtonText: { color: "#fff", fontSize: 15, fontWeight: "600" },

  clearButton: { alignSelf: "center", paddingVertical: 4 },
  clearButtonText: { fontSize: 13, color: "#999" },
});

export default App;
