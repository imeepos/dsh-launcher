import { useState } from "react";
import { addManualVersion } from "../api";
import { showSuccess } from "./toastStore";

// 手动添加对话框的提交状态机:成功后轻提醒并回调,失败保留行内错误。
function useManualAdd(onAdded: () => void, onClose: () => void) {
  const [bin, setBin] = useState("");
  const [cwd, setCwd] = useState("");
  const [id, setId] = useState("");
  const [tool, setTool] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit() {
    setSubmitting(true);
    setErr(null);
    try {
      const entry = await addManualVersion(
        bin,
        cwd.trim() ? cwd : null,
        id.trim() ? id : null,
        tool.trim() ? tool : null,
      );
      showSuccess("已添加版本 " + entry.id);
      onAdded();
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setSubmitting(false);
    }
  }

  return { bin, cwd, id, tool, err, submitting, setBin, setCwd, setId, setTool, submit };
}

export default useManualAdd;
