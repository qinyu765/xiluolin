import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";

import { ModelSettings } from "@/components/settings/ModelSettings";
import type { ProviderCatalog } from "@/generated/tauri-bindings";
import type { AppConfig } from "@/types";

Object.defineProperties(HTMLElement.prototype, {
  hasPointerCapture: { value: () => false, configurable: true },
  setPointerCapture: { value: () => undefined, configurable: true },
  releasePointerCapture: { value: () => undefined, configurable: true },
  scrollIntoView: { value: () => undefined, configurable: true },
});

const { catalog } = vi.hoisted(() => ({
  catalog: {
    asr: [
      {
        id: "local",
        name: "本地 Whisper",
        capability: "asr",
        protocol: "local-whisper",
        default_base_url: "",
        default_model: "ggml-base-q5_1.bin",
        fields: [],
        capabilities: {
          native_hotwords: false,
          max_hotwords: null,
          supports_prompt: true,
          max_duration_ms: null,
          local_model_management: true,
          max_language_hints: null,
        },
      },
      {
        id: "qwen-audio",
        name: "Qwen-Audio 3.0 ASR",
        capability: "asr",
        protocol: "dashscope-multimodal",
        default_base_url: "https://dashscope.aliyuncs.com",
        default_model: "qwen-audio-3.0-asr-flash",
        fields: [
          {
            key: "api_key",
            label: "API Key",
            kind: "api_key",
            required: true,
            secret: true,
            placeholder: "",
            help: "只保存到系统凭据库",
            choices: [],
            max_items: null,
          },
          {
            key: "language_hints",
            label: "语言提示",
            kind: "multi_select",
            required: false,
            secret: false,
            placeholder: "zh, en",
            help: "最多选择 4 种可能语言",
            choices: [],
            max_items: 4,
          },
        ],
        capabilities: {
          native_hotwords: true,
          max_hotwords: 100,
          supports_prompt: true,
          max_duration_ms: null,
          local_model_management: false,
          max_language_hints: 4,
        },
      },
      {
        id: "openai",
        name: "OpenAI-compatible ASR",
        capability: "asr",
        protocol: "multipart",
        default_base_url: "https://api.openai.com/v1",
        default_model: "whisper-1",
        fields: [],
        capabilities: {
          native_hotwords: false,
          max_hotwords: null,
          supports_prompt: true,
          max_duration_ms: null,
          local_model_management: false,
          max_language_hints: null,
        },
      },
    ],
    text: [
      {
        id: "openai-text",
        name: "OpenAI-compatible 文本",
        capability: "text",
        protocol: "openai-chat",
        default_base_url: "https://api.openai.com/v1",
        default_model: "gpt-4o-mini",
        fields: [],
        capabilities: {
          native_hotwords: false,
          max_hotwords: null,
          supports_prompt: false,
          max_duration_ms: null,
          local_model_management: false,
          max_language_hints: null,
        },
      },
    ],
  } as ProviderCatalog,
}));

vi.mock("@/generated/tauri-bindings", async (importOriginal) => {
  const original =
    await importOriginal<typeof import("@/generated/tauri-bindings")>();
  return {
    ...original,
    commands: {
      ...original.commands,
      listProviderCatalog: vi.fn().mockResolvedValue(catalog),
      realtimeAsrModelInfo: vi.fn().mockResolvedValue({
        name: "Zipformer 中英双语混合量化实验版",
        revision: "98590b7ed6443e77b714204da2757d75e1a642f4",
        path: "/models/realtime",
        state: "not_downloaded",
        enabled: false,
        total_size_bytes: 199_313_605,
        downloaded_size_bytes: 0,
      }),
    },
    events: {
      ...original.events,
      realtimeAsrDownloadProgress: {
        listen: vi.fn().mockResolvedValue(vi.fn()),
      },
    },
  };
});

vi.mock("@/components/settings/LocalAsrSettings", () => ({
  LocalAsrSettings: () => <div>本地模型管理</div>,
}));

function config(): AppConfig {
  return {
    config_version: 2,
    default_persona_id: "general",
    asr: {
      primary: "local",
      fallbacks: [],
      settings: {
        local: {
          api_key: "",
          base_url: "",
          model: "ggml-base-q5_1.bin",
          options: {},
        },
        "qwen-audio": {
          api_key: "",
          base_url: "https://dashscope.aliyuncs.com",
          model: "qwen-audio-3.0-asr-flash",
          options: {},
        },
      },
    },
    text: {
      primary: "openai-text",
      fallbacks: [],
      settings: {
        "openai-text": {
          api_key: "",
          base_url: "https://api.openai.com/v1",
          model: "gpt-4o-mini",
          options: {},
        },
      },
    },
    longpress_shortcut: "",
    toggle_shortcut: "",
    fn_hold_enabled: false,
    auto_save_history: true,
    mute_system_audio: false,
    selected_microphone: "",
    retain_recordings: false,
    realtime_preview_enabled: false,
  };
}

const requiredProps = {
  onConfigBlur: vi.fn(),
  onModelChanged: vi.fn(),
};

afterEach(cleanup);

test("catalog 驱动 Qwen 字段渲染", async () => {
  const user = userEvent.setup();
  const next = config();
  next.asr.primary = "qwen-audio";
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={next}
      updateConfig={vi.fn()}
    />,
  );

  expect(await screen.findByText("Qwen-Audio 3.0 ASR")).toBeInTheDocument();
  expect(screen.getAllByText("待补充").length).toBeGreaterThan(0);
  expect(screen.queryByLabelText(/API Key/)).not.toBeInTheDocument();
  expect(screen.queryByText("未配置备用 Provider")).not.toBeInTheDocument();
  await user.click(
    await screen.findByRole("button", { name: /Qwen-Audio 3\.0 ASR/ }),
  );
  expect(
    await screen.findByRole("heading", { name: "Qwen-Audio 3.0 ASR" }),
  ).toBeInTheDocument();
  expect(screen.getByLabelText(/API Key/)).toBeInTheDocument();
  expect(screen.getByLabelText("语言提示")).toBeInTheDocument();
  expect(screen.getByText("录音实时预览")).toBeInTheDocument();
});

test("local 加入云端 fallback 前要求隐私确认", async () => {
  const user = userEvent.setup();
  const updateConfig = vi.fn();
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={config()}
      updateConfig={updateConfig}
    />,
  );

  await user.click(await screen.findByRole("button", { name: /本地 Whisper/ }));
  await screen.findByText("本地模型管理");
  await user.click(
    screen.getByRole("combobox", { name: "添加 ASR 备用 Provider" }),
  );
  await user.click(
    await screen.findByRole("option", { name: "Qwen-Audio 3.0 ASR" }),
  );

  expect(confirm).toHaveBeenCalledWith(
    expect.stringContaining("音频会在本地识别失败时发送到云端"),
  );
  await waitFor(() => expect(updateConfig).not.toHaveBeenCalled());
  confirm.mockRestore();
});

test("将 fallback 切换为 primary 时自动去重", async () => {
  const user = userEvent.setup();
  const updateConfig = vi.fn();
  const next = config();
  next.asr.primary = "qwen-audio";
  next.asr.fallbacks = ["local"];
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={next}
      updateConfig={updateConfig}
    />,
  );

  await user.click(
    await screen.findByRole("button", { name: /Qwen-Audio 3\.0 ASR/ }),
  );
  await user.click(screen.getByRole("combobox", { name: "主 Provider" }));
  await user.click(await screen.findByRole("option", { name: "本地 Whisper" }));

  expect(updateConfig).toHaveBeenLastCalledWith(
    {
      asr: expect.objectContaining({ primary: "local", fallbacks: [] }),
    },
    "immediate",
  );
});

test("切换为 local primary 且保留云 fallback 时要求隐私确认", async () => {
  const user = userEvent.setup();
  const updateConfig = vi.fn();
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  const next = config();
  next.asr.primary = "qwen-audio";
  next.asr.fallbacks = ["local", "openai"];
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={next}
      updateConfig={updateConfig}
    />,
  );

  await user.click(
    await screen.findByRole("button", { name: /Qwen-Audio 3\.0 ASR/ }),
  );
  await user.click(screen.getByRole("combobox", { name: "主 Provider" }));
  await user.click(await screen.findByRole("option", { name: "本地 Whisper" }));

  expect(confirm).toHaveBeenCalledWith(
    expect.stringContaining("音频会在本地识别失败时发送到云端"),
  );
  expect(updateConfig).not.toHaveBeenCalled();
  confirm.mockRestore();
});

test("只在弹窗中显示所选 Provider 的完整配置", async () => {
  const user = userEvent.setup();
  const next = config();
  next.asr.primary = "local";
  next.asr.fallbacks = ["qwen-audio"];
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={next}
      updateConfig={vi.fn()}
    />,
  );

  await screen.findByRole("button", { name: /Qwen-Audio 3\.0 ASR/ });
  expect(screen.queryByLabelText(/API Key/)).not.toBeInTheDocument();
  await user.click(
    await screen.findByRole("button", { name: /Qwen-Audio 3\.0 ASR/ }),
  );

  expect(screen.getByLabelText(/API Key/)).toBeInTheDocument();
  expect(screen.getAllByText("备用 1").length).toBeGreaterThan(0);
  expect(screen.getByText("语言提示")).toBeInTheDocument();
});

test("Provider 弹窗保留可滚动内容区并提供更大的关闭按钮", async () => {
  const user = userEvent.setup();
  const next = config();
  next.asr.primary = "qwen-audio";
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={next}
      updateConfig={vi.fn()}
    />,
  );

  await user.click(
    await screen.findByRole("button", { name: /Qwen-Audio 3\.0 ASR/ }),
  );

  expect(screen.getByRole("dialog")).toHaveClass(
    "h-[min(90vh,48rem)]",
    "flex",
    "flex-col",
  );
  expect(screen.getByTestId("provider-editor-body")).toHaveClass(
    "flex-1",
    "overflow-y-auto",
  );
  expect(screen.getByRole("button", { name: "关闭" })).toHaveClass("size-9");
});

test("弹窗内调整备用顺序仍沿用原有路由更新", async () => {
  const user = userEvent.setup();
  const updateConfig = vi.fn();
  const next = config();
  next.asr.primary = "openai";
  next.asr.fallbacks = ["local", "qwen-audio"];
  render(
    <ModelSettings
      {...requiredProps}
      appConfig={next}
      updateConfig={updateConfig}
    />,
  );

  const manageButtons = await screen.findAllByRole("button", { name: "管理" });
  await user.click(manageButtons[0]);
  await user.click(screen.getByRole("button", { name: "上移 qwen-audio" }));

  expect(updateConfig).toHaveBeenLastCalledWith(
    {
      asr: expect.objectContaining({
        primary: "openai",
        fallbacks: ["qwen-audio", "local"],
      }),
    },
    "immediate",
  );
});
