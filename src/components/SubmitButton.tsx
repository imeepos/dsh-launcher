import Spinner from "./Spinner";

interface SubmitButtonProps {
  busy: boolean;
  label: string;
  busyLabel: string;
  className?: string;
  disabled?: boolean;
  type?: "button" | "submit";
  onClick?: () => void;
}

// 受控提交按钮:busy 时转圈 + busyLabel,用于对话框提交(异步逻辑归对话框所有)。
export default function SubmitButton({
  busy,
  label,
  busyLabel,
  className,
  disabled,
  type = "button",
  onClick,
}: SubmitButtonProps) {
  return (
    <button
      type={type}
      className={className}
      onClick={onClick}
      disabled={disabled || busy}
      aria-busy={busy}
    >
      {busy && <Spinner />}
      <span>{busy ? busyLabel : label}</span>
    </button>
  );
}
