import { useState } from "react";
import HomeFormDialog, { type HomeFormMode } from "./HomeFormDialog";
import HomeCard from "./HomeCard";
import ProfileCard from "./ProfileCard";
import StartControls from "./StartControls";
import useLaunchPad from "../hooks/useLaunchPad";

type LP = ReturnType<typeof useLaunchPad>;

function HomeSidebar({ lp, onAdd, onRegister }: { lp: LP; onAdd: () => void; onRegister: () => void }) {
  return (
    <div className="launch-col">
      <div className="col-label">
        <span>Homes</span>
        <button className="icon-add" onClick={onAdd} disabled={lp.busy} aria-label="新建 Home" title="新建 Home">
          +
        </button>
      </div>
      <div className="home-list">
        {lp.homes.map((h) => (
          <HomeCard key={h.id} home={h} selected={h.id === lp.homeId} onSelect={() => lp.setHomeId(h.id)} />
        ))}
        <button className="home-card add-card" onClick={onRegister}>登记已有 Home</button>
      </div>
    </div>
  );
}

function ProfileArea({ lp }: { lp: LP }) {
  return (
    <div className="launch-col">
      <div className="col-label">
        <span>
          Profiles
          {lp.home && (
            <>
              <span className="crumb-sep" aria-hidden>›</span>
              <span className="crumb-home mono">{lp.home.id}</span>
            </>
          )}
        </span>
      </div>
      {lp.error && <p className="error">{lp.error}</p>}
      {!lp.home ? (
        <div className="launch-placeholder">选择左侧 Home 查看 Profiles</div>
      ) : lp.profiles.length === 0 ? (
        <div className="launch-placeholder">该 Home 下还没有 profile</div>
      ) : (
        <div className="profile-grid">
          {lp.profiles.map((p) => {
            const key = lp.home!.id + "/" + p.name;
            return (
              <ProfileCard
                key={p.name}
                info={p}
                isRunning={lp.running.includes(key)}
                busy={lp.opBusy}
                onStart={() => lp.start(lp.home!, p, lp.opts.patch, lp.opts.args, lp.opts.cwd)}
                onStop={() => lp.stop(lp.home!, p)}
              />
            );
          })}
        </div>
      )}
    </div>
  );
}

function OptionsSection({ lp }: { lp: LP }) {
  return (
    <>
      <button className="link-toggle" onClick={() => lp.setOptionsOpen(!lp.optionsOpen)}>
        {lp.optionsOpen ? "隐藏启动选项" : "启动选项(patch / 参数 / cwd)"}
      </button>
      {lp.optionsOpen && (
        <StartControls
          homes={lp.homes}
          homeId={lp.homeId}
          onHome={lp.setHomeId}
          patch={lp.opts.patch}
          onPatch={(v) => lp.setOpts({ ...lp.opts, patch: v })}
          args={lp.opts.args}
          onArgs={(v) => lp.setOpts({ ...lp.opts, args: v })}
          cwd={lp.opts.cwd}
          onCwd={(v) => lp.setOpts({ ...lp.opts, cwd: v })}
          error={null}
        />
      )}
    </>
  );
}

// 启动台:默认视图,最高频操作(启动 profile)在这里一步完成。
export default function LaunchPad({ hidden }: { hidden?: boolean }) {
  const lp = useLaunchPad();
  const [form, setForm] = useState<{ mode: HomeFormMode } | null>(null);
  return (
    <div className="view" hidden={hidden}>
      <div className="view-head">
        <h2 className="page-title">启动台</h2>
        <p className="page-sub">选择 Home 与 Profile,一键启动</p>
      </div>
      <div className="launch-layout">
        <HomeSidebar
          lp={lp}
          onAdd={() => setForm({ mode: "create" })}
          onRegister={() => setForm({ mode: "add" })}
        />
        <ProfileArea lp={lp} />
        <OptionsSection lp={lp} />
      </div>
      {form && <HomeFormDialog mode={form.mode} onClose={() => setForm(null)} onDone={lp.refreshAll} />}
    </div>
  );
}
