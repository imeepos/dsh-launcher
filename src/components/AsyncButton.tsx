import { useCallback } from "react";
import useAsyncAction, { type AsyncStatus } from "../hooks/useAsyncAction";
import { showFailure, showSuccess } from "../hooks/toastStore";
import Spinner from "./Spinner";

interface AsyncButtonProps {
  task: () => Promise<void>;
  idle: string;
  loading: string;
  /** 成功微提醒:短暂替换按钮文案(配 ✓ 图标),超时自动恢复 idle。 */
  success?: string;
  /** 成功后额外弹出的轻提醒文案。 */
  successToast?: string;
  /** 失败轻提醒的前缀,toast 文案为「前缀: 错误」。 */
  failurePrefix?: string;
  className?: string;
  disabled?: boolean;
  title?: string;
}

const STATE_ICON: Record<Exclude<AsyncStatus, "idle" | "loading">, string> = {
  success: "✓",
  error: "✕",
};

// 自管理异步按钮:点击 -> loading(转圈+文案) -> 成功(✓+文案,短暂)/失败(✕+轻提醒) -> 恢复。
export default function AsyncButton({
  task,
  idle,
  loading,
  success,
  successToast,
  failurePrefix,
  className,
  disabled,
  title,
}: AsyncButtonProps) {
  const { status, run } = useAsyncAction();

  const onClick = useCallback(() => {
    void run(task).then((result) => {
      if (result.ok) {
        if (successToast) showSuccess(successToast);
        return;
      }
      if (failurePrefix) showFailure(failurePrefix, result.error);
    });
  }, [run, task, successToast, failurePrefix]);

  const stateClass =
    status === "idle" ? "" : " " + status;
  const label =
    status === "loading" ? loading : status === "success" && success ? success : idle;
  const terminal = status === "success" || status === "error";

  return (
    <button
      type="button"
      className={(className ?? "") + stateClass}
      onClick={onClick}
      disabled={disabled || status === "loading"}
      aria-busy={status === "loading"}
      title={title}
    >
      {status === "loading" && <Spinner />}
      {terminal && <span className="state-icon">{STATE_ICON[status]}</span>}
      <span>{label}</span>
    </button>
  );
}
