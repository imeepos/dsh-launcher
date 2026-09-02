export default function ManualAddFields({
  bin,
  cwd,
  id,
  tool,
  onBinChange,
  onCwdChange,
  onIdChange,
  onToolChange,
}: {
  bin: string;
  cwd: string;
  id: string;
  tool: string;
  onBinChange: (v: string) => void;
  onCwdChange: (v: string) => void;
  onIdChange: (v: string) => void;
  onToolChange: (v: string) => void;
}) {
  return (
    <>
      <label>
        bin(可执行文件,必填,支持 ~)
        <input
          value={bin}
          onChange={(e) => onBinChange(e.target.value)}
          placeholder="~/.local/bin/dsh"
          autoFocus
        />
      </label>
      <label>
        cwd(可选)
        <input
          value={cwd}
          onChange={(e) => onCwdChange(e.target.value)}
          placeholder="运行时工作目录"
        />
      </label>
      <label>
        工具名(可选,空视为 dsh)
        <input
          value={tool}
          onChange={(e) => onToolChange(e.target.value)}
          placeholder="releasectl"
        />
      </label>
      <label>
        id(可选,默认 manual-&lt;bin 文件名&gt;)
        <input value={id} onChange={(e) => onIdChange(e.target.value)} placeholder="my-dsh" />
      </label>
    </>
  );
}
