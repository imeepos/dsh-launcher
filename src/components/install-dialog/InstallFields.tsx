export function NpmFields({
  version,
  onVersionChange,
  disabled,
}: {
  version: string;
  onVersionChange: (v: string) => void;
  disabled: boolean;
}) {
  return (
    <>
      <label>
        DSH 版本号
        <input
          value={version}
          onChange={(e) => onVersionChange(e.target.value)}
          placeholder="0.1.1-rc.2"
          disabled={disabled}
        />
      </label>
      <p className="hint">
        执行 npm install --prefix ~/.dsh-launcher/versions/v&lt;版本&gt; @deepseek-ai/dsh@&lt;版本&gt;
      </p>
    </>
  );
}

export function DevFields({
  repoPath,
  onRepoPathChange,
  disabled,
  placeholder,
}: {
  repoPath: string;
  onRepoPathChange: (v: string) => void;
  disabled: boolean;
  placeholder: string;
}) {
  return (
    <>
      <label>
        repo checkout 路径
        <input
          value={repoPath}
          onChange={(e) => onRepoPathChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
        />
      </label>
      <p className="hint">登记为 dev 版本:启动命令 pnpm dsh,cwd=repo 路径</p>
    </>
  );
}
