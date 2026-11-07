import { useState, useCallback } from "react";
import { Terminal, Check, Loader2, Download, Zap, Shield, GitBranch, Package, ChevronRight, Bot, Sparkles, Code, Cpu } from "lucide-react";
import { cn } from "../lib/utils";

const isTauri = "__TAURI_INTERNALS__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    if (cmd === "check_tool") {
      return { installed: false, current_version: "", latest_version: "", needs_update: false } as T;
    }
    return { success: true } as T;
  }
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

type Status = "pending" | "checking" | "installed" | "not_installed" | "installing" | "upgrading" | "done" | "error";

interface Step {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  status: Status;
  currentVersion?: string;
  latestVersion?: string;
  needsUpdate?: boolean;
  error?: string;
}

const initialSteps: Step[] = [
  {
    id: "claude",
    name: "Claude Code",
    description: "Anthropic 的 AI 编程助手",
    icon: <Terminal className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "codex",
    name: "OpenAI Codex",
    description: "OpenAI 的本地编程代理",
    icon: <Bot className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "gemini",
    name: "Gemini CLI",
    description: "Google 的 AI 编程助手",
    icon: <Sparkles className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "opencode",
    name: "OpenCode",
    description: "开源终端 AI 编程工具",
    icon: <Code className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "openclaw",
    name: "OpenClaw",
    description: "AI 编程助手",
    icon: <Cpu className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "hermes",
    name: "Hermes",
    description: "Nous Research 的 AI 代理",
    icon: <Sparkles className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "mimo",
    name: "MiMo Code",
    description: "小米的 AI 编程助手",
    icon: <Cpu className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "ccswitch",
    name: "CC-Switch",
    description: "AI 工具管理器，统一管理多个 AI 工具",
    icon: <Package className="w-5 h-5" />,
    status: "pending",
  },
  {
    id: "kilocode",
    name: "Kilo Code",
    description: "VS Code AI 编程扩展",
    icon: <Code className="w-5 h-5" />,
    status: "pending",
  },
];

export function Installer() {
  const [steps, setSteps] = useState<Step[]>(initialSteps);
  const [currentStep, setCurrentStep] = useState<number>(-1);
  const [allDone, setAllDone] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [showLogs, setShowLogs] = useState(false);

  const addLog = useCallback((msg: string) => {
    setLogs((prev) => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  }, []);

  const updateStep = useCallback((index: number, update: Partial<Step>) => {
    setSteps((prev) => prev.map((s, i) => (i === index ? { ...s, ...update } : s)));
  }, []);

  const checkTool = useCallback(
    async (toolId: string): Promise<{ installed: boolean; currentVersion: string; latestVersion: string; needsUpdate: boolean }> => {
      try {
        const result = await invoke<{ installed: boolean; current_version: string; latest_version: string; needs_update: boolean }>("check_tool", { tool: toolId });
        return {
          installed: result.installed,
          currentVersion: result.current_version,
          latestVersion: result.latest_version,
          needsUpdate: result.needs_update,
        };
      } catch {
        return { installed: false, currentVersion: "", latestVersion: "", needsUpdate: false };
      }
    },
    []
  );

  const installTool = useCallback(
    async (toolId: string): Promise<{ success: boolean; error?: string }> => {
      try {
        const result = await invoke<{ success: boolean; error?: string }>("install_tool", { tool: toolId });
        return result;
      } catch (e) {
        return { success: false, error: String(e) };
      }
    },
    []
  );

  const checkAll = useCallback(async () => {
    setLogs([]);
    setShowLogs(true);

    for (let i = 0; i < steps.length; i++) {
      updateStep(i, { status: "checking", error: undefined });
      addLog(`正在检测 ${steps[i].name}...`);

      const check = await checkTool(steps[i].id);
      addLog(`${steps[i].name}: ${check.installed ? `已安装 (${check.currentVersion})` : "未安装"}${check.needsUpdate ? `，有新版本 ${check.latestVersion}` : ""}`);

      if (check.installed) {
        updateStep(i, {
          status: "installed",
          currentVersion: check.currentVersion,
          latestVersion: check.latestVersion,
          needsUpdate: check.needsUpdate,
        });
      } else {
        updateStep(i, { status: "not_installed" });
      }
    }

    addLog("检测完成!");
  }, [steps, updateStep, addLog, checkTool]);

  const installSingle = useCallback(async (index: number) => {
    const step = steps[index];
    updateStep(index, { status: "installing", error: undefined });
    addLog(`正在安装 ${step.name}...`);

    // 先确保 Git 和 Node.js 已安装
    const gitCheck = await checkTool("git");
    if (!gitCheck.installed) {
      addLog("正在安装 Git...");
      await installTool("git");
    }
    const nodeCheck = await checkTool("node");
    if (!nodeCheck.installed) {
      addLog("正在安装 Node.js...");
      await installTool("node");
    }

    const result = await installTool(step.id);
    if (result.success) {
      const newCheck = await checkTool(step.id);
      updateStep(index, {
        status: "done",
        currentVersion: newCheck.currentVersion,
        latestVersion: newCheck.latestVersion,
        needsUpdate: false,
      });
      addLog(`${step.name} 安装完成`);
    } else {
      updateStep(index, { status: "error", error: result.error || "安装失败" });
      addLog(`${step.name} 安装失败: ${result.error}`);
    }
  }, [steps, updateStep, addLog, checkTool, installTool]);

  const upgradeSingle = useCallback(async (index: number) => {
    const step = steps[index];
    updateStep(index, { status: "upgrading", error: undefined });
    addLog(`正在升级 ${step.name}...`);

    const result = await installTool(step.id);
    if (result.success) {
      const newCheck = await checkTool(step.id);
      updateStep(index, {
        status: "done",
        currentVersion: newCheck.currentVersion,
        latestVersion: newCheck.latestVersion,
        needsUpdate: false,
      });
      addLog(`${step.name} 升级完成`);
    } else {
      updateStep(index, { status: "error", error: result.error || "升级失败" });
      addLog(`${step.name} 升级失败: ${result.error}`);
    }
  }, [steps, updateStep, addLog, checkTool, installTool]);

  const openTool = useCallback(async (index: number) => {
    const step = steps[index];
    addLog(`正在打开 ${step.name}...`);
    try {
      await invoke("open_tool", { tool: step.id });
      addLog(`${step.name} 已打开`);
    } catch (e) {
      addLog(`打开 ${step.name} 失败: ${e}`);
    }
  }, [steps, addLog]);

  const uninstallToolFn = useCallback(async (index: number) => {
    const step = steps[index];
    updateStep(index, { status: "installing", error: undefined });
    addLog(`正在卸载 ${step.name}...`);

    try {
      const result = await invoke<{ success: boolean; error?: string }>("uninstall_tool", { tool: step.id });
      if (result.success) {
        updateStep(index, {
          status: "not_installed",
          currentVersion: undefined,
          latestVersion: undefined,
          needsUpdate: false,
        });
        addLog(`${step.name} 卸载完成`);
      } else {
        updateStep(index, { status: "error", error: result.error || "卸载失败" });
        addLog(`${step.name} 卸载失败: ${result.error}`);
      }
    } catch (e) {
      updateStep(index, { status: "error", error: String(e) });
      addLog(`${step.name} 卸载失败: ${e}`);
    }
  }, [steps, updateStep, addLog]);

  const installAll = useCallback(async () => {
    setAllDone(false);
    setLogs([]);
    setShowLogs(true);

    // 先确保 Git 和 Node.js 已安装
    addLog("正在检查基础依赖...");
    const gitCheck = await checkTool("git");
    if (!gitCheck.installed) {
      addLog("正在安装 Git...");
      await installTool("git");
    }
    const nodeCheck = await checkTool("node");
    if (!nodeCheck.installed) {
      addLog("正在安装 Node.js...");
      await installTool("node");
    }

    for (let i = 0; i < steps.length; i++) {
      setCurrentStep(i);
      updateStep(i, { status: "checking", error: undefined });
      addLog(`正在检测 ${steps[i].name}...`);

      const check = await checkTool(steps[i].id);
      addLog(`${steps[i].name}: ${check.installed ? `已安装 (${check.currentVersion})` : "未安装"}`);

      if (check.installed) {
        updateStep(i, {
          status: "installed",
          currentVersion: check.currentVersion,
          latestVersion: check.latestVersion,
          needsUpdate: check.needsUpdate,
        });
        continue;
      }

      updateStep(i, { status: "installing" });
      addLog(`正在安装 ${steps[i].name}...`);

      const result = await installTool(steps[i].id);
      if (result.success) {
        const newCheck = await checkTool(steps[i].id);
        updateStep(i, {
          status: "done",
          currentVersion: newCheck.currentVersion,
          latestVersion: newCheck.latestVersion,
          needsUpdate: false,
        });
        addLog(`${steps[i].name} 安装完成`);
      } else {
        updateStep(i, { status: "error", error: result.error || "安装失败" });
        addLog(`${steps[i].name} 安装失败: ${result.error}`);
        break;
      }
    }

    setCurrentStep(-1);
    setAllDone(true);
    addLog("所有步骤完成!");
  }, [steps, updateStep, addLog, checkTool, installTool]);

  const getStatusIcon = (status: Status) => {
    switch (status) {
      case "checking":
      case "installing":
      case "upgrading":
        return <Loader2 className="w-4 h-4 text-accent animate-spin-slow" />;
      case "installed":
      case "done":
        return <Check className="w-4 h-4 text-emerald-400" />;
      case "error":
        return <span className="w-4 h-4 text-red-400 text-center text-xs font-bold">!</span>;
      case "not_installed":
        return <Download className="w-4 h-4 text-surface-4" />;
      default:
        return <div className="w-4 h-4 rounded-full border-2 border-surface-4" />;
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-3 border-b border-surface-3 bg-surface-1/50 backdrop-blur-sm">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-amber-700 flex items-center justify-center">
            <Terminal className="w-4 h-4 text-surface-0" />
          </div>
          <div>
            <h1 className="text-sm font-semibold text-white">AI 工具安装器</h1>
            <p className="text-[10px] text-gray-400">一键安装 AI 编程环境</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={checkAll}
            disabled={steps.some(s => s.status === "checking" || s.status === "installing" || s.status === "upgrading")}
            className={cn(
              "px-3 py-1.5 rounded-lg text-xs font-medium transition-all",
              steps.some(s => s.status === "checking" || s.status === "installing" || s.status === "upgrading")
                ? "bg-surface-3 text-surface-4 cursor-not-allowed"
                : "bg-surface-2 text-white hover:bg-surface-3 border border-surface-3"
            )}
          >
            <span className="flex items-center gap-1.5">
              <Shield className="w-3.5 h-3.5" />
              检测版本
            </span>
          </button>
          <button
            onClick={installAll}
            disabled={currentStep >= 0}
            className={cn(
              "px-4 py-1.5 rounded-lg text-xs font-medium transition-all",
              currentStep >= 0
                ? "bg-surface-3 text-surface-4 cursor-not-allowed"
                : allDone
                ? "bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30"
                : "bg-gradient-to-r from-accent to-amber-600 text-surface-0 hover:from-accent-hover hover:to-amber-500"
            )}
          >
            <span className="flex items-center gap-1.5">
              {currentStep >= 0 ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin-slow" />
                  安装中...
                </>
              ) : allDone ? (
                <>
                  <Check className="w-3.5 h-3.5" />
                  完成
                </>
              ) : (
                <>
                  <Download className="w-3.5 h-3.5" />
                  全部安装
                </>
              )}
            </span>
          </button>
        </div>
      </header>

      {/* Main Content - Left Right Layout */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Side - Tool List */}
        <div className="flex-1 overflow-y-auto p-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
            {steps.map((step, index) => (
              <div
                key={step.id}
                className={cn(
                  "relative rounded-lg border p-3 transition-all duration-200 hover:border-accent/30",
                  step.status === "checking" || step.status === "installing" || step.status === "upgrading"
                    ? "border-accent/50 bg-accent/5"
                    : step.status === "installed" || step.status === "done"
                    ? "border-emerald-500/30 bg-emerald-500/5"
                    : step.status === "error"
                    ? "border-red-500/30 bg-red-500/5"
                    : "border-surface-3 bg-surface-2/50"
                )}
              >
                <div className="flex items-center gap-3">
                  <div
                    className={cn(
                      "w-9 h-9 rounded-lg flex items-center justify-center shrink-0 transition-colors",
                      step.status === "installed" || step.status === "done"
                        ? "bg-emerald-500/20 text-emerald-400"
                        : step.status === "error"
                        ? "bg-red-500/20 text-red-400"
                        : step.status === "checking" || step.status === "installing" || step.status === "upgrading"
                        ? "bg-accent/20 text-accent"
                        : "bg-surface-3 text-gray-400"
                    )}
                  >
                    {step.icon}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-white">{step.name}</span>
                      {step.currentVersion && (
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-3 text-gray-300 font-mono">
                          v{step.currentVersion}
                        </span>
                      )}
                    </div>
                    {step.currentVersion && step.latestVersion && step.needsUpdate && (
                      <p className="text-[10px] text-amber-400 mt-0.5">
                        最新: v{step.latestVersion}
                      </p>
                    )}
                    <p className="text-[11px] text-gray-400 mt-0.5 truncate">{step.description}</p>
                    {step.error && <p className="text-[10px] text-red-400 mt-1">{step.error}</p>}
                  </div>
                  <div className="shrink-0 flex flex-col items-end gap-1.5">
                    {getStatusIcon(step.status)}
                    {step.status === "not_installed" && (
                      <button
                        onClick={() => installSingle(index)}
                        className="px-2.5 py-1 text-[10px] rounded-md bg-accent text-surface-0 hover:bg-accent-hover transition-colors"
                      >
                        安装
                      </button>
                    )}
                    {(step.status === "installed" || step.status === "done") && (
                      <div className="flex gap-1.5">
                        {step.needsUpdate && (
                          <button
                            onClick={() => upgradeSingle(index)}
                            className="px-2.5 py-1 text-[10px] rounded-md bg-amber-500 text-surface-0 hover:bg-amber-600 transition-colors"
                          >
                            升级
                          </button>
                        )}
                        <button
                          onClick={() => openTool(index)}
                          className="px-2.5 py-1 text-[10px] rounded-md bg-emerald-600 text-surface-0 hover:bg-emerald-700 transition-colors"
                        >
                          打开
                        </button>
                        <button
                          onClick={() => uninstallToolFn(index)}
                          className="px-2.5 py-1 text-[10px] rounded-md bg-red-600 text-surface-0 hover:bg-red-700 transition-colors"
                        >
                          卸载
                        </button>
                      </div>
                    )}
                  </div>
                </div>

                {/* Progress bar */}
                {(step.status === "checking" || step.status === "installing" || step.status === "upgrading") && (
                  <div className="mt-2 h-0.5 bg-surface-3 rounded-full overflow-hidden">
                    <div className="h-full bg-accent rounded-full animate-pulse-glow w-2/3" />
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Right Side - Log Panel */}
        {logs.length > 0 && (
          <div className="w-72 border-l border-surface-3 bg-surface-1/80 flex flex-col">
            <div className="px-4 py-2.5 border-b border-surface-3 flex items-center justify-between">
              <span className="text-xs font-medium text-gray-300 flex items-center gap-1.5">
                <Shield className="w-3.5 h-3.5" />
                安装日志
              </span>
              <button
                onClick={() => setShowLogs(!showLogs)}
                className="text-gray-400 hover:text-white transition-colors"
              >
                <ChevronRight className={cn("w-3.5 h-3.5 transition-transform", !showLogs && "rotate-180")} />
              </button>
            </div>
            {showLogs && (
              <div className="flex-1 overflow-y-auto p-3">
                {logs.map((log, i) => (
                  <div key={i} className="font-mono text-[10px] text-gray-400 py-0.5 leading-relaxed">
                    {log}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
